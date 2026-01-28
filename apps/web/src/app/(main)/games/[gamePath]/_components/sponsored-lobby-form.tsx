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
import { formatAmount } from "@/lib/utils";
import { ApiClient } from "@/lib/api/client";
import { toast } from "sonner";
import type { Lobby, CreateLobbyRequest, Game, Token } from "@/lib/definitions";
import { waitForTxConfirmed } from "@/lib/contract-utils/waitForTxConfirmed";
import { deployStacksContract } from "@/lib/contract-utils/deploy";
import { joinSponsoredContract } from "@/lib/contract-utils/join";
import type {
	AssetString,
	ContractIdString,
} from "@stacks/connect/dist/types/methods";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { useUser, useUserLoading } from "@/lib/stores/user";
import { useAppActions } from "@/lib/stores/app";
import { Loader2 } from "lucide-react";

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

type SponsoredLobbyFormValues = z.infer<typeof sponsoredLobbySchema>;

interface SponsoredLobbyFormProps {
	tokens: Token[];
	minimumAmount: number;
	getDefaultDescription: () => string;
	setSelectedToken: (value: string) => void;
	game: Game;
}

export default function SponsoredLobbyForm({
	tokens,
	minimumAmount,
	getDefaultDescription,
	setSelectedToken,
	game,
}: SponsoredLobbyFormProps) {
	const router = useRouter();
	const user = useUser();
	const { setLobbyCreationProgress, clearLobbyCreationProgress } =
		useAppActions();
	const isUserLoading = useUserLoading();
	const isAuthenticated = !isUserLoading && user;
	const form = useForm<SponsoredLobbyFormValues>({
		resolver: zodResolver(sponsoredLobbySchema),
		defaultValues: {
			lobbyName: "",
			description: "",
			lobbyType: "public",
			poolAmount: "",
			selectedToken: "stx",
		},
	});
	const [error, setError] = useState<string | null>(null);
	const [progress, setProgress] = useState<string | null>(null);

	const handleSubmit = async (values: SponsoredLobbyFormValues) => {
		setError(null);
		setProgress(null);
		try {
			const amount = parseFloat(values.poolAmount);
			if (amount < minimumAmount) {
				setError(`Pool amount must be at least ${minimumAmount}`);
				return;
			}
			const selectedTokenObj = tokens.find(
				(t) => t.contractId === values.selectedToken
			);
			const payload: CreateLobbyRequest = {
				name: values.lobbyName,
				description:
					values.description?.trim() || getDefaultDescription(),
				gameId: game.id,
				gamePath: game.path,
				isPrivate: values.lobbyType === "private",
				isSponsored: true,
				entryAmount: 0,
				currentAmount: amount,
				tokenSymbol: selectedTokenObj?.name,
				tokenContractId: selectedTokenObj?.contractId,
			};
			try {
				setProgress("Setting up your contract");
				const contractResponse = await ApiClient.get<string>(
					`/api/sponsored-contract?gameCreatorId=${game.creatorId}&poolSize=${amount}&contractId=${values.selectedToken}`
				);
				if (contractResponse.error) {
					toast.error("Failed to set up contract");
					console.error(contractResponse.error);
					setProgress(null);
					return;
				}
				const deployResult = await deployStacksContract({
					clarityCode: contractResponse.data!,
					tokenName: selectedTokenObj!.name,
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
				setLobbyCreationProgress({
					contractAddress: deployResult.contractAddress,
					step: "deployed",
					payload: {
						...payload,
						contractAddress: deployResult.contractAddress,
					},
				});
				const joinTxId = await joinSponsoredContract({
					contract: deployResult.contractAddress as ContractIdString,
					amount,
					isCreator: true,
					tokenId:
						`${selectedTokenObj!.contractId}::${selectedTokenObj!.name}` as AssetString,
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
				await waitForTxConfirmed(joinTxId);
				setLobbyCreationProgress({
					contractAddress: deployResult.contractAddress,
					step: "joined",
					payload: {
						...payload,
						contractAddress: deployResult.contractAddress,
					},
				});
			} catch (error) {
				setError("Failed to deploy or join contract");
				setProgress(null);
				console.error(error);
				return;
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
					name="lobbyName"
					render={({ field }) => (
						<FormItem>
							<FormLabel>
								Lobby Name{" "}
								<span className="text-destructive">*</span>
							</FormLabel>
							<FormControl>
								<Input
									placeholder="Enter lobby name"
									{...field}
									maxLength={50}
								/>
							</FormControl>
							<FormDescription>
								Choose a descriptive name for your lobby (max 50
								characters)
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>
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
				<div className="flex gap-2">
					<FormField
						control={form.control}
						name="poolAmount"
						render={({ field }) => (
							<FormItem className="flex-1">
								<FormLabel>
									Pool Amount{" "}
									<span className="text-destructive">*</span>
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
						control={form.control}
						name="selectedToken"
						render={({ field }) => (
							<FormItem className="self-end">
								<FormLabel className="sr-only">Token</FormLabel>
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
													<span>{token.name}</span>
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
					{minimumAmount.toFixed(2)} ≈ $10
				</FormDescription>
				{error && <p className="text-sm text-destructive">{error}</p>}
				{isUserLoading ? (
					<Skeleton className="flex justify-self-end w-full sm:w-fit rounded-full h-13 sm:min-w-30" />
				) : isAuthenticated ? (
					<Button
						type="submit"
						className="flex justify-self-end w-full sm:w-fit rounded-full"
						disabled={form.formState.isSubmitting}
					>
						{form.formState.isSubmitting && (
							<Loader2 className="mr-2 h-4 w-4 animate-spin inline-block align-middle" />
						)}
						{form.formState.isSubmitting
							? progress || "Creating..."
							: "Create Lobby"}
					</Button>
				) : (
					<Button
						type="button"
						className="flex justify-self-end w-full sm:w-fit rounded-full"
						asChild
					>
						<Link href="/login">Login to Create a Lobby</Link>
					</Button>
				)}
			</form>
		</Form>
	);
}
