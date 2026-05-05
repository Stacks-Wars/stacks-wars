import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
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
import { Skeleton } from "@/components/ui/skeleton";
import Link from "next/link";
import { ApiClient } from "@/lib/api/client";
import { toast } from "sonner";
import type { Lobby, CreateLobbyRequest, Game } from "@/lib/definitions";
import {
	ExpectedError,
	waitForTxConfirmed,
} from "@/lib/contract-utils/waitForTxConfirmed";
import { deployStacksContract } from "@/lib/contract-utils/deploy";
import { joinNormalContract } from "@/lib/contract-utils/join";
import { useState } from "react";
import { useUser, useUserLoading } from "@/lib/stores/user";
import { useRouter } from "next/navigation";
import { useAppActions } from "@/lib/stores/app";
import { Loader2 } from "lucide-react";
import type { ContractIdString } from "@stacks/transactions";

const normalLobbySchema = z.object({
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

type NormalLobbyFormValues = z.infer<typeof normalLobbySchema>;

interface NormalLobbyFormProps {
	getDefaultLobbyName: () => string;
	getDefaultDescription: () => string;
	game: Game;
}

export default function NormalLobbyForm({
	getDefaultLobbyName,
	getDefaultDescription,
	game,
}: NormalLobbyFormProps) {
	const router = useRouter();
	const user = useUser();
	const { setLobbyCreationProgress, clearLobbyCreationProgress } =
		useAppActions();
	const isUserLoading = useUserLoading();
	const isAuthenticated = !isUserLoading && user;
	const form = useForm<NormalLobbyFormValues>({
		// @ts-ignore - Zod v4 compatibility issue with @hookform/resolvers
		resolver: zodResolver(normalLobbySchema),
		defaultValues: {
			description: "",
			lobbyType: "public",
			entryAmount: "",
		},
	});
	const [error, setError] = useState<string | null>(null);
	const [progress, setProgress] = useState<string | null>(null);

	const handleSubmit = async (values: NormalLobbyFormValues) => {
		setError(null);
		setProgress(null);
		try {
			const payload: CreateLobbyRequest = {
				name: getDefaultLobbyName(),
				description:
					values.description?.trim() || getDefaultDescription(),
				gameId: game.id,
				gamePath: game.path,
				isPrivate: values.lobbyType === "private",
				isSponsored: false,
			};
			if (values.entryAmount) {
				const amount = parseFloat(values.entryAmount);
				payload.entryAmount = amount;
				payload.currentAmount = amount;
				payload.tokenContractId = "stx";
				payload.tokenSymbol = "STX";
				try {
					setProgress("Setting up your contract");
					const contractResponse = await ApiClient.get<string>(
						`/api/contract?gameCreatorId=${game.creatorId}&entryFee=${amount}&contractId=stx`
					);
					if (contractResponse.error) {
						toast.error("Failed to set up contract");
						console.error(contractResponse.error);
						setProgress(null);
						return;
					}
					const deployResult = await deployStacksContract({
						clarityCode: contractResponse.data!,
						tokenName: "stx",
					});
					if (!deployResult.txid) {
						toast.error("Failed to deploy contract", {
							description: "Please try again later.",
						});
						setProgress(null);
						throw new Error(
							"Failed to deploy contract: No transaction ID returned"
						);
					}
					setProgress("Deploying your contract");
					await waitForTxConfirmed(deployResult.txid);
					const contractAddress = `${user?.walletAddress}.${deployResult.name}`;
					payload.contractAddress = contractAddress;
					setLobbyCreationProgress({
						contractAddress,
						step: "deployed",
						payload: {
							...payload,
						},
					});
					const joinTxId = await joinNormalContract({
						contract: contractAddress as ContractIdString,
						amount,
						address: user!.walletAddress,
					});
					if (!joinTxId) {
						toast.error("Failed to join contract", {
							description: "Please try again later.",
						});
						setProgress(null);
						throw new Error(
							"Failed to join contract: No transaction ID returned"
						);
					}
					setProgress("Adding you to the contract");
					await waitForTxConfirmed(
						joinTxId,
						ExpectedError.ERR_ALREADY_JOINED
					);
					setLobbyCreationProgress({
						contractAddress,
						step: "joined",
						payload: {
							...payload,
						},
					});
				} catch (error) {
					setError("Failed to deploy or join contract");
					setProgress(null);
					console.error(error);
					return;
				}
			}
			setProgress("Creating your lobby");
			const response = await ApiClient.post<Lobby>("/api/lobby", payload);
			if (response.error) {
				toast.error("Failed to create lobby", {
					description: "Please try again later.",
				});
				setProgress(null);
				console.error("API error:", response.error);
				return;
			}
			if (response.data) {
				clearLobbyCreationProgress();
				setProgress(null);
				toast.success("Lobby created successfully!");
				router.push(`/room/${response.data.path}`);
			}
		} catch (err) {
			setProgress(null);
			console.error(err);
		}
	};

	return (
		<Form {...form}>
			<form
				onSubmit={form.handleSubmit(handleSubmit)}
				className="space-y-6"
			>
				<FormField
					control={form.control}
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
					control={form.control}
					name="lobbyType"
					render={({ field }) => (
						<FormItem>
							<FormLabel>
								Lobby Type{" "}
								<span className="text-destructive">*</span>
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
								Public lobbies are open to everyone. Private
								lobbies require creator approval
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>
				<FormField
					control={form.control}
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
							<FormDescription>Minimum 5 STX.</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>
				{error && <p className="text-destructive text-sm">{error}</p>}
				{isUserLoading ? (
					<Skeleton className="flex h-13 w-full justify-self-end rounded-full sm:w-fit sm:min-w-30" />
				) : isAuthenticated ? (
					<Button
						type="submit"
						className="flex w-full justify-self-end rounded-full sm:w-fit"
						disabled={form.formState.isSubmitting}
					>
						{form.formState.isSubmitting && (
							<Loader2 className="mr-2 inline-block h-4 w-4 animate-spin align-middle" />
						)}
						{form.formState.isSubmitting
							? progress || "Creating..."
							: "Create Lobby"}
					</Button>
				) : (
					<Button
						type="button"
						className="flex w-full justify-self-end rounded-full sm:w-fit"
						asChild
					>
						<Link href="/login">Login to Create a Lobby</Link>
					</Button>
				)}
			</form>
		</Form>
	);
}
