"use client";

import { useState, useEffect } from "react";
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
import { useUser, useUserLoading } from "@/lib/stores/user";
import type {
	Game,
	Lobby,
	CreateLobbyRequest,
	Token,
	TokenInfo,
} from "@/lib/definitions";
import { useRouter } from "next/navigation";
import { displayUserIdentifier, formatAmount } from "@/lib/utils";
import { deployStacksContract } from "@/lib/wallet";
import { toast } from "sonner";
import { Skeleton } from "@/components/ui/skeleton";
import Link from "next/link";

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
	selectedToken: z.string().min(1, "Token is required"),
});

type NormalLobbyFormValues = z.infer<typeof normalLobbySchema>;
type SponsoredLobbyFormValues = z.infer<typeof sponsoredLobbySchema>;

export default function CreateLobbyForm(game: Game) {
	const user = useUser();
	const isUserLoading = useUserLoading();
	const isAuthenticated = !isUserLoading && user;
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
			selectedToken: "stx",
		},
	});

	const [tokens, setTokens] = useState<Token[]>([]);
	const [minimumAmount, setMinimumAmount] = useState<number>(0);

	useEffect(() => {
		if (isAuthenticated && user?.walletAddress) {
			ApiClient.get<Token[]>(`/api/balance/${user.walletAddress}`).then(
				(response) => {
					if (response.data) {
						const fetchedTokens = response.data;
						const hasSTX = fetchedTokens.some(
							(t) => t.contractId === "stx"
						);
						if (!hasSTX) {
							fetchedTokens.unshift({
								name: "STX",
								balance: 0,
								contractId: "stx",
							});
						}
						setTokens(fetchedTokens);
					}
				}
			);
		} else {
			// Default to STX when not authenticated
			setTokens([{ name: "STX", balance: 0, contractId: "stx" }]);
		}
	}, [isAuthenticated, user]);

	const [selectedToken, setSelectedToken] = useState<string>("stx");

	useEffect(() => {
		if (selectedToken) {
			ApiClient.get<TokenInfo>(`/api/token/${selectedToken}`).then(
				(response) => {
					if (response.data) {
						setMinimumAmount(response.data.minimumAmount);
					}
				}
			);
		}
	}, [selectedToken]);

	const getDefaultDescription = () => {
		const userIdentifier = user ? displayUserIdentifier(user) : "Anonymous";
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

				// Get and deploy contract
				try {
					const contractResponse = await ApiClient.get<string>(
						`/api/contract?gameCreatorId=${game.creatorId}&entryFee=${amount}&contractId=stx`
					);
					if (contractResponse.error) {
						toast.error("Failed to get contract template");
						setError("Failed to get contract template");
						return;
					}

					const deployResult = await deployStacksContract({
						clarityCode: contractResponse.data!,
						tokenName: "stx",
					});

					payload.contractAddress = deployResult.contractAddress;
				} catch (deployError) {
					toast.error("Failed to deploy contract", {
						description:
							deployError instanceof Error
								? deployError.message
								: undefined,
					});
					setError("Failed to deploy contract");
					return;
				}
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
			if (amount < minimumAmount) {
				setError(`Pool amount must be at least ${minimumAmount}`);
				return;
			}
			payload.entryAmount = 0; // No entry fee for sponsored lobbies
			payload.currentAmount = amount; // Sponsor funds the whole pool

			const selectedToken = tokens.find(
				(t) => t.contractId === values.selectedToken
			);
			if (selectedToken) {
				payload.tokenSymbol = selectedToken.name;
				payload.tokenContractId = selectedToken.contractId;
			}

			// Get and deploy contract
			try {
				const contractResponse = await ApiClient.get<string>(
					`/api/sponsored-contract?gameCreatorId=${game.creatorId}&poolSize=${amount}&contractId=${values.selectedToken}`
				);
				if (contractResponse.error) {
					toast.error("Failed to get contract template");
					setError("Failed to get contract template");
					return;
				}

				const deployResult = await deployStacksContract({
					clarityCode: contractResponse.data!,
					tokenName: selectedToken!.name,
				});

				payload.contractAddress = deployResult.contractAddress;
			} catch (deployError) {
				toast.error("Failed to deploy contract", {
					description:
						deployError instanceof Error
							? deployError.message
							: undefined,
				});
				setError("Failed to deploy contract");
				return;
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

						{isUserLoading ? (
							<Skeleton className="flex justify-self-end w-full sm:w-fit rounded-full h-13 sm:min-w-30" />
						) : isAuthenticated ? (
							<Button
								type="submit"
								className="flex justify-self-end w-full sm:w-fit rounded-full"
								disabled={normalForm.formState.isSubmitting}
							>
								{normalForm.formState.isSubmitting
									? "Creating..."
									: "Create Lobby"}
							</Button>
						) : (
							<Button
								type="button"
								className="flex justify-self-end w-full sm:w-fit rounded-full"
								asChild
							>
								<Link href="/login">
									Login to Create a Lobby
								</Link>
							</Button>
						)}
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

						<div className="flex gap-2">
							<FormField
								control={sponsoredForm.control}
								name="poolAmount"
								render={({ field }) => (
									<FormItem className="flex-1">
										<FormLabel>
											Pool Amount{" "}
											<span className="text-destructive">
												*
											</span>
										</FormLabel>
										<FormControl>
											<Input
												type="number"
												placeholder="0"
												{...field}
												step="0.01"
											/>
										</FormControl>
										<FormMessage />
									</FormItem>
								)}
							/>
							<FormField
								control={sponsoredForm.control}
								name="selectedToken"
								render={({ field }) => (
									<FormItem className="self-end">
										<FormLabel className="sr-only">
											Token
										</FormLabel>
										<Select
											onValueChange={(value) => {
												field.onChange(value);
												setSelectedToken(value);
											}}
											defaultValue={field.value}
										>
											<FormControl>
												<SelectTrigger className="w-40">
													<SelectValue placeholder="Token" />
												</SelectTrigger>
											</FormControl>
											<SelectContent>
												{tokens.map((token) => (
													<SelectItem
														key={token.contractId}
														value={token.contractId}
													>
														<div className="flex items-center justify-between w-full">
															<span>
																{token.name}
															</span>
															<span className="text-xs ml-4 text-foreground/70 font-mono">
																(
																{formatAmount(
																	token.balance
																)}
																)
															</span>
														</div>
													</SelectItem>
												))}
											</SelectContent>
										</Select>
										<FormMessage />
									</FormItem>
								)}
							/>
						</div>
						<FormDescription>
							The total prize pool you will fund. Minimum:{" "}
							{minimumAmount.toFixed(2)}
						</FormDescription>

						{error && (
							<p className="text-sm text-destructive">{error}</p>
						)}

						{isUserLoading ? (
							<Skeleton className="flex justify-self-end w-full sm:w-fit rounded-full h-13 sm:min-w-30" />
						) : isAuthenticated ? (
							<Button
								type="submit"
								className="flex justify-self-end w-full sm:w-fit rounded-full"
								disabled={sponsoredForm.formState.isSubmitting}
							>
								{sponsoredForm.formState.isSubmitting
									? "Creating..."
									: "Create Lobby"}
							</Button>
						) : (
							<Button
								type="button"
								className="flex justify-self-end w-full sm:w-fit rounded-full"
								asChild
							>
								<Link href="/login">
									Login to Create a Lobby
								</Link>
							</Button>
						)}
					</form>
				</Form>
			</TabsContent>
		</Tabs>
	);
}
