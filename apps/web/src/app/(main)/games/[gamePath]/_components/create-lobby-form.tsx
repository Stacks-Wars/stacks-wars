"use client";

import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useUser, useUserLoading } from "@/lib/stores/user";
import type { Game } from "@/lib/definitions";
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

	const getDefaultDescription = () => {
		const userIdentifier = user ? displayUserIdentifier(user) : "Anonymous";
		return `Join ${userIdentifier}'s ${game.name} lobby!`;
	};

	const handleContinueCreation = async () => {
		if (!user) return;
		const response = await handleContinue(user.walletAddress);
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
			{progress?.restoredFromStorage && isAuthenticated ? (
				<div className="bg-card mb-6 w-full space-y-4 rounded-3xl border p-4 sm:space-y-6 sm:p-6 lg:p-8">
					<div className="space-y-3 sm:space-y-4">
						<div className="flex items-center justify-between gap-2">
							<p className="truncate text-base font-semibold sm:text-lg lg:text-xl">
								Resume Lobby Creation
							</p>
							<span className="inline-block rounded-full bg-yellow-100 px-3 py-1 text-xs font-semibold text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
								In Progress
							</span>
						</div>
						<p className="text-muted-foreground text-xs sm:text-sm lg:text-base">
							You have an incomplete lobby creation in progress.
						</p>
						<div className="flex items-center gap-2 text-xs sm:text-sm">
							<span className="bg-muted rounded px-2 py-1 font-mono">
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
					<TabsList className="grid w-full grid-cols-2 gap-2 rounded-full p-1 sm:p-2.5">
						<TabsTrigger
							value="normal"
							className="data-[state=active]:bg-primary/50 rounded-full py-2 text-xs sm:py-2.5 sm:text-lg"
						>
							Normal
						</TabsTrigger>
						<TabsTrigger
							value="sponsored"
							className="data-[state=active]:bg-primary/50 rounded-full py-2 text-xs sm:py-2.5 sm:text-lg"
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
							getDefaultDescription={getDefaultDescription}
							game={game}
						/>
					</TabsContent>
				</Tabs>
			)}
		</>
	);
}
