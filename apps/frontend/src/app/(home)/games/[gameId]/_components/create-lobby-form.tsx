"use client";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Loader } from "lucide-react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, useWatch } from "react-hook-form";
import * as z from "zod";
import {
	Form,
	FormControl,
	FormDescription,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { apiRequest, ApiRequestProps } from "@/lib/api";
import { toast } from "sonner";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { getClaimFromJwt } from "@/lib/getClaimFromJwt";
import { nanoid } from "nanoid";
import { createGamePool, transferFee } from "@/lib/actions/createGamePool";
import { joinGamePool } from "@/lib/actions/joinGamePool";
import { waitForTxConfirmed } from "@/lib/actions/waitForTxConfirmed";
import { useConnectUser } from "@/contexts/ConnectWalletContext";
import {
	saveLobbyRecoveryData,
	updateLobbyRecoveryData,
	deleteLobbyRecoveryData,
	getLobbyRecoveryData,
	type LobbyRecoveryData,
} from "@/lib/db/lobby-recovery";
import LobbyRecoveryCard from "@/components/ui/lobby-recovery-card";

const formSchema = z.object({
	name: z.string().min(3, {
		message: "Lobby name must be at least 3 characters.",
	}),
	description: z
		.string()
		.min(3, {
			message: "Lobby description must be at least 3 characters.",
		})
		.optional(),
	withPool: z.boolean(),
	amount: z
		.number()
		.min(5, {
			message: "Amount must be at least 5 STX.",
		})
		.optional(),
});

interface FormData {
	name: string;
	description?: string;
	withPool: boolean;
	amount?: number;
}

interface CreateLobbyFormProps {
	gameId: string;
}

export default function CreateLobbyForm({ gameId }: CreateLobbyFormProps) {
	const router = useRouter();
	const [isLoading, setIsLoading] = useState(false);
	const { isConnecting, isConnected, handleConnect } = useConnectUser();
	const [deployedContract, setDeployedContract] = useState<{
		contractName: string;
		contractAddress: `${string}.${string}`;
		entryAmount: number;
		txId: string;
	} | null>(null);
	const [joined, setJoined] = useState<{
		contractAddress: `${string}.${string}`;
		txId: string;
		entryAmount: number;
	} | null>(null);
	const [recoveryData, setRecoveryData] = useState<LobbyRecoveryData | null>(
		null
	);
	const [showRecovery, setShowRecovery] = useState(false);
	const recoveryIdRef = useRef<string | null>(null);
	const prevAmountRef = useRef<number | undefined>(undefined);

	const form = useForm<FormData>({
		resolver: zodResolver(formSchema),
		defaultValues: {
			name: "",
			withPool: true,
		},
		mode: "onChange",
	});

	// Check for recovery data on component mount
	useEffect(() => {
		const checkRecoveryData = async () => {
			try {
				const userAddress = await getClaimFromJwt<string>("wallet");
				if (!userAddress) return;

				const data = await getLobbyRecoveryData(userAddress, gameId);
				if (data) {
					setRecoveryData(data);
					setShowRecovery(true);
					recoveryIdRef.current = data.id;

					// Restore deployed contract state if exists
					if (data.deployedContract) {
						setDeployedContract({
							contractName: data.deployedContract.contractName,
							contractAddress:
								data.deployedContract.contractAddress,
							entryAmount: data.deployedContract.entryAmount,
							txId: data.deployedContract.txId,
						});
					}

					// Restore joined state if exists
					if (data.joinedContract) {
						setJoined({
							contractAddress:
								data.joinedContract.contractAddress,
							txId: data.joinedContract.txId,
							entryAmount: data.joinedContract.entryAmount,
						});
					}
				}
			} catch (error) {
				console.error("Error checking recovery data:", error);
			}
		};

		checkRecoveryData();
	}, [gameId]);

	//const amount = form.watch("amount");
	const amount = useWatch({
		control: form.control,
		name: "amount",
	});

	useEffect(() => {
		const amountChanged =
			prevAmountRef.current !== undefined &&
			amount !== prevAmountRef.current;

		if (deployedContract && amountChanged) {
			console.warn("⚠️ Amount changed, resetting deployed contract");
			setDeployedContract(null);
			setJoined(null);
		}

		prevAmountRef.current = amount;
	}, [deployedContract, amount]);

	const withPool = form.watch("withPool");

	// Recovery handlers
	const handleContinueRecovery = async () => {
		if (!recoveryData) return;

		// Restore form values
		form.reset({
			name: recoveryData.formData.name,
			description: recoveryData.formData.description,
			withPool: recoveryData.formData.withPool,
			amount: recoveryData.formData.amount,
		});

		setShowRecovery(false);

		// If we're at the deployed stage, continue with joining
		if (
			recoveryData.status === "deployed" &&
			recoveryData.deployedContract &&
			!recoveryData.joinedContract
		) {
			// Set the deployed contract state and continue to join
			const contractInfo = {
				contractName: recoveryData.deployedContract.contractName,
				contractAddress: recoveryData.deployedContract.contractAddress,
				entryAmount: recoveryData.deployedContract.entryAmount,
				txId: recoveryData.deployedContract.txId,
			};
			setDeployedContract(contractInfo);

			// Trigger the join process
			setTimeout(() => {
				form.handleSubmit(onSubmit)();
			}, 100);
		} else if (
			recoveryData.status === "joined" &&
			recoveryData.deployedContract &&
			recoveryData.joinedContract
		) {
			// Set both deployed and joined contract states for completed recovery
			const contractInfo = {
				contractName: recoveryData.deployedContract.contractName,
				contractAddress: recoveryData.deployedContract.contractAddress,
				entryAmount: recoveryData.deployedContract.entryAmount,
				txId: recoveryData.deployedContract.txId,
			};
			setDeployedContract(contractInfo);

			const joinInfo = {
				contractAddress: recoveryData.joinedContract.contractAddress,
				txId: recoveryData.joinedContract.txId,
				entryAmount: recoveryData.joinedContract.entryAmount,
			};
			setJoined(joinInfo);

			// Continue to lobby creation
			setTimeout(() => {
				form.handleSubmit(onSubmit)();
			}, 100);
		} else if (recoveryData.status === "pending") {
			setTimeout(() => {
				form.handleSubmit(onSubmit)();
			}, 100);
		}
	};

	const handleDiscardRecovery = async () => {
		if (recoveryData?.id) {
			await deleteLobbyRecoveryData(recoveryData.id);
		}
		setRecoveryData(null);
		setShowRecovery(false);
		setDeployedContract(null);
		setJoined(null);
		recoveryIdRef.current = null;
	};

	const onSubmit = async (values: FormData) => {
		setIsLoading(true);

		try {
			let apiParams: ApiRequestProps;

			if (values.withPool) {
				if (!values.amount) {
					throw new Error("Amount is required when using a pool");
				}

				const deployerAddress = await getClaimFromJwt<string>("wallet");
				if (!deployerAddress) {
					toast.error("You need to connect your wallet first.");
					setIsLoading(false);
					return;
				}

				// Save initial recovery data if not continuing from recovery
				if (!recoveryIdRef.current) {
					const recoveryId = await saveLobbyRecoveryData({
						userAddress: deployerAddress,
						gameId,
						formData: values,
						status: "pending",
					});
					recoveryIdRef.current = recoveryId;
				}

				let contractInfo = deployedContract;

				if (!contractInfo) {
					const contractName = `${nanoid(5)}-stacks-wars`;
					const contract: `${string}.${string}` = `${deployerAddress}.${contractName}`;
					const entry_amount = values.amount;

					const deployTx = await createGamePool(
						entry_amount,
						contractName,
						deployerAddress
					);

					if (!deployTx.txid) {
						throw new Error(
							"Failed to deploy game pool: missing transaction ID"
						);
					}

					try {
						await waitForTxConfirmed(deployTx.txid);
					} catch (err) {
						console.error("❌ TX failed or aborted:", err);
						throw err;
					}

					contractInfo = {
						contractName,
						contractAddress: contract,
						entryAmount: entry_amount,
						txId: deployTx.txid,
					};
					setDeployedContract(contractInfo);

					// Update recovery data with deployed contract info
					if (recoveryIdRef.current) {
						await updateLobbyRecoveryData(recoveryIdRef.current, {
							deployedContract: {
								contractName,
								contractAddress: contract,
								entryAmount: entry_amount,
								txId: deployTx.txid,
							},
							status: "deployed",
						});
					}
				}

				let joinInfo = joined;
				let tx_id: string;

				if (
					joinInfo &&
					joinInfo.contractAddress === contractInfo.contractAddress
				) {
					tx_id = joinInfo.txId;
				} else {
					const joinTxId = await joinGamePool(
						contractInfo.contractAddress,
						contractInfo.entryAmount
					);

					if (!joinTxId) {
						throw new Error(
							"Failed to join game pool: missing transaction ID"
						);
					}

					try {
						await waitForTxConfirmed(joinTxId);
					} catch (err) {
						console.error("❌ TX failed or aborted:", err);
						throw err;
					}

					joinInfo = {
						contractAddress: contractInfo.contractAddress,
						txId: joinTxId,
						entryAmount: contractInfo.entryAmount,
					};
					setJoined(joinInfo);

					// Update recovery data with joined contract info
					if (recoveryIdRef.current) {
						await updateLobbyRecoveryData(recoveryIdRef.current, {
							joinedContract: {
								contractAddress: contractInfo.contractAddress,
								txId: joinTxId,
								entryAmount: contractInfo.entryAmount,
							},
							status: "joined",
						});
					}

					tx_id = joinTxId;
				}

				apiParams = {
					path: "/lobby",
					method: "POST",
					body: {
						name: values.name,
						description: values.description || null,
						entry_amount: contractInfo.entryAmount,
						current_amount: contractInfo.entryAmount,
						contract_address: contractInfo.contractAddress,
						tx_id,
						game_id: gameId,
					},
					tag: "lobby",
					revalidateTag: "lobby",
					revalidatePath: "/lobby",
				};
			} else {
				const feeTransferTx = await transferFee();

				if (!feeTransferTx.txid) {
					throw new Error(
						"Failed to transfer fee: missing transaction ID"
					);
				}

				try {
					await waitForTxConfirmed(feeTransferTx.txid);
				} catch (err) {
					console.error("❌ Fee transfer failed or aborted:", err);
					throw err;
				}

				apiParams = {
					path: "/lobby",
					method: "POST",
					body: {
						name: values.name,
						description: values.description || null,
						game_id: gameId,
						tx_id: feeTransferTx.txid,
					},
					tag: "lobby",
					revalidateTag: "lobby",
					revalidatePath: "/lobby",
				};
			}
			const lobbyId = await apiRequest<string>(apiParams);

			// Delete recovery data
			if (recoveryIdRef.current) {
				await deleteLobbyRecoveryData(recoveryIdRef.current);
			}

			setDeployedContract(null);
			setJoined(null);
			setRecoveryData(null);
			setShowRecovery(false);
			recoveryIdRef.current = null;

			toast.info("Please wait while we redirect you to your lobby");
			router.replace(`/lobby/${lobbyId}`);
		} catch (error) {
			toast.error("Something went wrong", {
				description: "Please try again",
			});
			console.error("An error occured while creating lobby:", error);
		} finally {
			setIsLoading(false);
		}
	};

	return (
		<div className="space-y-6">
			{showRecovery && recoveryData && (
				<LobbyRecoveryCard
					recoveryData={recoveryData}
					onContinue={handleContinueRecovery}
					onDiscard={handleDiscardRecovery}
					isSponsored={false}
				/>
			)}

			<Card className="bg-primary/30">
				<CardHeader>
					<CardTitle className="text-2xl">Create a Lobby</CardTitle>
					<CardDescription>
						Set up a new lobby and invite friends to join
					</CardDescription>
				</CardHeader>
				<Form {...form}>
					<form
						onSubmit={form.handleSubmit(onSubmit)}
						className="space-y-4"
					>
						<CardContent className="space-y-4">
							<FormField
								control={form.control}
								name="name"
								render={({ field }) => (
									<FormItem>
										<FormLabel className="inline-flex items-center gap-1">
											Lobby Name
											<span className="text-destructive">
												*
											</span>
										</FormLabel>
										<FormControl>
											<Input
												placeholder="Enter lobby name"
												{...field}
											/>
										</FormControl>
										<FormDescription>
											Give your lobby a descriptive name
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
											<Input
												placeholder="Enter lobby description"
												{...field}
											/>
										</FormControl>
										<FormDescription>
											Provide additional details about
											your lobby
										</FormDescription>
										<FormMessage />
									</FormItem>
								)}
							/>

							<FormField
								control={form.control}
								name="withPool"
								render={({ field }) => (
									<FormItem className="flex flex-row items-center space-x-2 space-y-0">
										<FormControl>
											<Switch
												checked={field.value}
												onCheckedChange={field.onChange}
											/>
										</FormControl>
										<FormLabel>
											Create lobby with pool
										</FormLabel>
									</FormItem>
								)}
							/>
							{withPool && (
								<FormField
									control={form.control}
									name="amount"
									render={({ field }) => (
										<FormItem>
											<FormLabel className="inline-flex items-center gap-1">
												Pool Amount (STX)
												<span className="text-destructive">
													*
												</span>
											</FormLabel>
											<FormControl>
												<Input
													type="number"
													placeholder="Enter amount in STX"
													value={field.value || ""}
													onChange={(e) => {
														const value =
															e.target.value;
														field.onChange(
															value === ""
																? undefined
																: parseFloat(
																		value
																	)
														);
													}}
												/>
											</FormControl>
											<FormDescription>
												This is the initial amount
												you&apos;ll contribute to the
												pool and entry fee for other
												players
											</FormDescription>
											<FormMessage />
										</FormItem>
									)}
								/>
							)}
						</CardContent>
						<CardFooter className="flex justify-end">
							{!isConnected ? (
								<Button
									onClick={handleConnect}
									type="button"
									disabled={isConnecting}
								>
									{isConnecting && (
										<Loader
											className="h-4 w-4 mr-1 animate-spin"
											size={17}
										/>
									)}
									{isConnecting
										? "Connecting..."
										: "Connect wallet to create lobby"}
								</Button>
							) : (
								<Button type="submit" disabled={isLoading}>
									{isLoading && (
										<Loader
											className="h-4 w-4 mr-1 animate-spin"
											size={17}
										/>
									)}
									{isLoading
										? "Creating..."
										: deployedContract
											? "Join Deployed Lobby"
											: "Create Lobby"}
								</Button>
							)}
						</CardFooter>
					</form>
				</Form>
			</Card>
		</div>
	);
}
