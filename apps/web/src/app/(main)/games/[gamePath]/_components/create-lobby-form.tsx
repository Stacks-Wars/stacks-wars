"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import {
	Form,
	FormControl,
	FormDescription,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { ApiClient } from "@/lib/api/client";
import { useUserStore } from "@/lib/stores/user";
import type { Game, Lobby, CreateLobbyRequest } from "@/lib/definitions";
import { useRouter } from "next/navigation";
import { formatAddress } from "@/lib/wallet";

// Zod schemas
const normalLobbySchema = z.object({
	lobbyName: z
		.string()
		.min(1, "Lobby name is required")
		.max(50, "Lobby name must be at most 50 characters"),
	description: z
		.string()
		.max(200, "Description must be at most 200 characters")
		.optional(),
	lobbyType: z.enum(["public", "private"]),
	entryAmount: z
		.string()
		.optional()
		.refine(
			(val) => {
				if (!val || val === "") return true;
				const num = parseFloat(val);
				return !isNaN(num) && num >= 5;
			},
			{ message: "Pool amount must be at least 5 STX" }
		),
});

const sponsoredLobbySchema = z.object({
	lobbyName: z
		.string()
		.min(1, "Lobby name is required")
		.max(50, "Lobby name must be at most 50 characters"),
	description: z
		.string()
		.max(200, "Description must be at most 200 characters")
		.optional(),
	lobbyType: z.enum(["public", "private"]),
	poolAmount: z
		.string()
		.min(1, "Pool amount is required")
		.refine((val) => !isNaN(parseFloat(val)) && parseFloat(val) > 0, {
			message: "Pool amount must be greater than 0",
		}),
	tokenSymbol: z
		.string()
		.min(1, "Token symbol is required")
		.max(10, "Token symbol must be at most 10 characters"),
	tokenContractId: z.string().optional(),
});

type NormalLobbyFormValues = z.infer<typeof normalLobbySchema>;
type SponsoredLobbyFormValues = z.infer<typeof sponsoredLobbySchema>;

export default function CreateLobbyForm(game: Game) {
	const { user } = useUserStore();
	const [error, setError] = useState<string | null>(null);
	const router = useRouter();

	// Normal lobby form
	const normalForm = useForm<NormalLobbyFormValues>({
		resolver: zodResolver(normalLobbySchema),
		defaultValues: {
			lobbyName: "",
			description: "",
			lobbyType: "public",
			entryAmount: "",
		},
	});

	// Sponsored lobby form
	const sponsoredForm = useForm<SponsoredLobbyFormValues>({
		resolver: zodResolver(sponsoredLobbySchema),
		defaultValues: {
			lobbyName: "",
			description: "",
			lobbyType: "public",
			poolAmount: "",
			tokenSymbol: "STX",
			tokenContractId: "",
		},
	});

	const getDefaultDescription = () => {
		const userIdentifier =
			user?.username ||
			user?.displayName ||
			(user?.walletAddress && formatAddress(user.walletAddress)) ||
			"Anonymous";
		return `Join ${userIdentifier}'s ${game.name} lobby!`;
	};

	const handleNormalSubmit = async (values: NormalLobbyFormValues) => {
		setError(null);

		try {
			const payload: CreateLobbyRequest = {
				name: values.lobbyName,
				description:
					values.description?.trim() || getDefaultDescription(),
				gameId: game.id,
				gamePath: game.path,
				isPrivate: values.lobbyType === "private",
				isSponsored: false,
			};

			// Add pool amount if provided
			if (values.entryAmount) {
				const amount = parseFloat(values.entryAmount);
				payload.entryAmount = amount;
				payload.currentAmount = amount;
				payload.tokenSymbol = "STX";
			}

			const response = await ApiClient.post<Lobby>("/api/lobby", payload);

			if (response.error) {
				setError(response.error);
				return;
			}

			// Redirect to lobby page
			if (response.data) {
				router.push(`/room/${response.data.path}`);
			}
		} catch (err) {
			setError("Failed to create lobby");
		}
	};

	const handleSponsoredSubmit = async (values: SponsoredLobbyFormValues) => {
		setError(null);

		try {
			const payload: CreateLobbyRequest = {
				name: values.lobbyName,
				description:
					values.description?.trim() || getDefaultDescription(),
				gameId: game.id,
				gamePath: game.path,
				isPrivate: values.lobbyType === "private",
				isSponsored: true,
			};

			const amount = parseFloat(values.poolAmount);
			payload.entryAmount = 0; // No entry fee for sponsored lobbies
			payload.currentAmount = amount; // Sponsor funds the whole pool
			payload.tokenSymbol = values.tokenSymbol;

			if (values.tokenContractId?.trim()) {
				payload.tokenContractId = values.tokenContractId;
			}

			const response = await ApiClient.post<Lobby>("/api/lobby", payload);

			if (response.error) {
				setError(response.error);
				return;
			}

			// Redirect to lobby page
			if (response.data) {
				router.push(`/room/${response.data.path}`);
			}
		} catch (err) {
			setError("Failed to create lobby");
		}
	};

	return (
		<Tabs defaultValue="normal" className="w-full">
			<TabsList className="grid w-full grid-cols-2 gap-2 p-1 sm:p-2.5 rounded-full">
				<TabsTrigger
					value="normal"
					className="data-[state=active]:bg-primary/50 text-xs sm:text-lg py-2 sm:py-2.5 rounded-full"
				>
					Normal
				</TabsTrigger>
				<TabsTrigger
					value="sponsored"
					className="data-[state=active]:bg-primary/50 text-xs sm:text-lg py-2 sm:py-2.5 rounded-full"
				>
					Sponsored
				</TabsTrigger>
			</TabsList>

			{/* Normal Lobby Tab */}
			<TabsContent value="normal" className="mt-4 sm:mt-8">
				<Form {...normalForm}>
					<form
						onSubmit={normalForm.handleSubmit(handleNormalSubmit)}
						className="space-y-6"
					>
						<FormField
							control={normalForm.control}
							name="lobbyName"
							render={({ field }) => (
								<FormItem>
									<FormLabel>
										Lobby Name{" "}
										<span className="text-destructive">
											*
										</span>
									</FormLabel>
									<FormControl>
										<Input
											placeholder="Enter lobby name"
											{...field}
											maxLength={50}
										/>
									</FormControl>
									<FormDescription>
										Choose a descriptive name for your lobby
										(max 50 characters)
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						<FormField
							control={normalForm.control}
							name="description"
							render={({ field }) => (
								<FormItem>
									<FormLabel>Description</FormLabel>
									<FormControl>
										<Textarea
											placeholder={getDefaultDescription()}
											{...field}
											maxLength={200}
											rows={3}
										/>
									</FormControl>
									<FormDescription>
										Add details about your lobby. (max 200
										characters)
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						<FormField
							control={normalForm.control}
							name="lobbyType"
							render={({ field }) => (
								<FormItem>
									<FormLabel>
										Lobby Type{" "}
										<span className="text-destructive">
											*
										</span>
									</FormLabel>
									<Select
										onValueChange={field.onChange}
										defaultValue={field.value}
									>
										<FormControl>
											<SelectTrigger className="w-full">
												<SelectValue placeholder="Select lobby type" />
											</SelectTrigger>
										</FormControl>
										<SelectContent>
											<SelectItem value="public">
												Public
											</SelectItem>
											<SelectItem value="private">
												Private
											</SelectItem>
										</SelectContent>
									</Select>
									<FormDescription>
										Public lobbies are open to everyone.
										Private lobbies require creator approval
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						<FormField
							control={normalForm.control}
							name="entryAmount"
							render={({ field }) => (
								<FormItem>
									<FormLabel>Entry Fee (STX)</FormLabel>
									<FormControl>
										<Input
											type="number"
											placeholder="0"
											{...field}
											min="5"
											step="0.01"
										/>
									</FormControl>
									<FormDescription>
										Minimum 5 STX.
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						{error && (
							<p className="text-sm text-destructive">{error}</p>
						)}

						<Button
							type="submit"
							className="flex justify-self-end w-fit rounded-full"
							disabled={normalForm.formState.isSubmitting}
						>
							{normalForm.formState.isSubmitting
								? "Creating..."
								: "Create Lobby"}
						</Button>
					</form>
				</Form>
			</TabsContent>

			{/* Sponsored Lobby Tab */}
			<TabsContent value="sponsored" className="mt-8">
				<Form {...sponsoredForm}>
					<form
						onSubmit={sponsoredForm.handleSubmit(
							handleSponsoredSubmit
						)}
						className="space-y-6"
					>
						<FormField
							control={sponsoredForm.control}
							name="lobbyName"
							render={({ field }) => (
								<FormItem>
									<FormLabel>
										Lobby Name{" "}
										<span className="text-destructive">
											*
										</span>
									</FormLabel>
									<FormControl>
										<Input
											placeholder="Enter lobby name"
											{...field}
											maxLength={50}
										/>
									</FormControl>
									<FormDescription>
										Choose a descriptive name for your lobby
										(max 50 characters)
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						<FormField
							control={sponsoredForm.control}
							name="description"
							render={({ field }) => (
								<FormItem>
									<FormLabel>Description</FormLabel>
									<FormControl>
										<Textarea
											placeholder={getDefaultDescription()}
											{...field}
											maxLength={200}
											rows={3}
										/>
									</FormControl>
									<FormDescription>
										Add details about your lobby. (max 200
										characters)
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						<FormField
							control={sponsoredForm.control}
							name="lobbyType"
							render={({ field }) => (
								<FormItem>
									<FormLabel>
										Lobby Type{" "}
										<span className="text-destructive">
											*
										</span>
									</FormLabel>
									<Select
										onValueChange={field.onChange}
										defaultValue={field.value}
									>
										<FormControl>
											<SelectTrigger className="w-full">
												<SelectValue placeholder="Select lobby type" />
											</SelectTrigger>
										</FormControl>
										<SelectContent>
											<SelectItem value="public">
												Public
											</SelectItem>
											<SelectItem value="private">
												Private
											</SelectItem>
										</SelectContent>
									</Select>
									<FormDescription>
										Public lobbies are open to everyone.
										Private lobbies require creator approval
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						<FormField
							control={sponsoredForm.control}
							name="poolAmount"
							render={({ field }) => (
								<FormItem>
									<FormLabel>
										Pool Amount{" "}
										<span className="text-destructive">
											*
										</span>
									</FormLabel>
									<FormControl>
										<div className="flex gap-2">
											<Input
												type="number"
												placeholder="0"
												{...field}
												min="0"
												step="0.01"
												className="flex-1"
											/>
											<FormField
												control={sponsoredForm.control}
												name="tokenSymbol"
												render={({
													field: symbolField,
												}) => (
													<Input
														placeholder="STX"
														{...symbolField}
														maxLength={10}
														className="w-24"
													/>
												)}
											/>
										</div>
									</FormControl>
									<FormDescription>
										The total prize pool you will fund.
										Players join for free (entry fee = 0)
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						{error && (
							<p className="text-sm text-destructive">{error}</p>
						)}

						<Button
							type="submit"
							className="flex justify-self-end w-fit rounded-full"
							disabled={sponsoredForm.formState.isSubmitting}
						>
							{sponsoredForm.formState.isSubmitting
								? "Creating..."
								: "Create Lobby"}
						</Button>
					</form>
				</Form>
			</TabsContent>
		</Tabs>
	);
}
