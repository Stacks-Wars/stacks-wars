"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ApiClient } from "@/lib/api/client";
import { useUser, useUserLoading } from "@/lib/stores/user";
import type { Game, Token, TokenInfo } from "@/lib/definitions";
import { useRouter } from "next/navigation";
import { displayUserIdentifier } from "@/lib/utils";
import { toast } from "sonner";
import { useLobbyCreationProgress, useAppActions } from "@/lib/stores/app";
import NormalLobbyForm from "./normal-lobby-form";
import SponsoredLobbyForm from "./sponsored-lobby-form";

export default function CreateLobbyForm(game: Game) {
	const user = useUser();
	const isUserLoading = useUserLoading();
	const isAuthenticated = !isUserLoading && user;
	const router = useRouter();
	const progress = useLobbyCreationProgress();
	const { clearLobbyCreationProgress, handleContinue } = useAppActions();

	const [tokens, setTokens] = useState<Token[]>([]);
	const [minimumAmount, setMinimumAmount] = useState<number>(0);
	const [selectedToken, setSelectedToken] = useState<string>("stx");

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

	const handleContinueCreation = async () => {
		const response = await handleContinue(user!.walletAddress);
		if (response.error) {
			console.error("API error:", response.error);
			return;
		}
		if (response.data) {
			clearLobbyCreationProgress();
			toast.success("Lobby created successfully!");
			router.push(`/room/${response.data.path}`);
		}
	};

	return (
		<>
			{progress?.restoredFromStorage ? (
				<div className="bg-card border p-4 sm:p-6 lg:p-8 rounded-3xl w-full space-y-4 sm:space-y-6 mb-6">
					<div className="space-y-3 sm:space-y-4">
						<div className="flex items-center justify-between gap-2">
							<p className="truncate text-base sm:text-lg lg:text-xl font-semibold">
								Resume Lobby Creation
							</p>
							<span className="inline-block bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 rounded-full px-3 py-1 text-xs font-semibold">
								In Progress
							</span>
						</div>
						<p className="text-xs sm:text-sm lg:text-base text-muted-foreground">
							You have an incomplete lobby creation in progress.
						</p>
						<div className="flex items-center gap-2 text-xs sm:text-sm">
							<span className="font-mono bg-muted px-2 py-1 rounded">
								{progress.contractAddress}
							</span>
							{progress.step === "deployed" && (
								<span className="text-yellow-700 dark:text-yellow-300">
									(waiting to join)
								</span>
							)}
							{progress.step === "joined" && (
								<span className="text-green-700 dark:text-green-300">
									(ready to post lobby)
								</span>
							)}
						</div>
					</div>
					<div className="flex gap-2 pt-2">
						<Button
							className="rounded-full px-6 py-2 text-sm font-medium"
							variant="secondary"
							onClick={() => {
								handleContinueCreation();
							}}
						>
							Continue
						</Button>
						<Button
							className="rounded-full px-6 py-2 text-sm font-medium"
							variant="outline"
							onClick={() => {
								clearLobbyCreationProgress();
								toast.info("Lobby creation progress discarded");
							}}
						>
							Discard
						</Button>
					</div>
				</div>
			) : (
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

					<TabsContent value="normal" className="mt-4 sm:mt-8">
						<NormalLobbyForm
							getDefaultDescription={getDefaultDescription}
							game={game}
						/>
					</TabsContent>

					<TabsContent value="sponsored" className="mt-8">
						<SponsoredLobbyForm
							tokens={tokens}
							minimumAmount={minimumAmount}
							getDefaultDescription={getDefaultDescription}
							setSelectedToken={setSelectedToken}
							game={game}
						/>
					</TabsContent>
				</Tabs>
			)}
		</>
	);
}
