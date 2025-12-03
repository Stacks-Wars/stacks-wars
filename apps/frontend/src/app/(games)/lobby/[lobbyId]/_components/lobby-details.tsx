import { Card, CardContent, CardTitle, CardHeader } from "@/components/ui/card";
import { Info, Timer, User, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { LobbyClientMessage } from "@/hooks/useLobbySocket";
import { useState, useEffect } from "react";
import { toast } from "sonner";
import { EXPLORER_BASE_URL } from "@/lib/constants";
import Link from "next/link";
import { truncateAddress } from "@/lib/utils";
import { Lobby, lobbyState } from "@/types/schema/lobby";
import { Player } from "@/types/schema/player";

interface LobbyDetailsProps {
	lobby: Lobby;
	players: Player[];
	countdown: number | null;
	lobbyState: lobbyState;
	sendMessage: (msg: LobbyClientMessage) => Promise<void>;
	userId: string;
	isKicking: boolean;
	onLeaveCheck?: (callback: (isConnected: boolean) => void) => void;
	cachedPlayerConnectionStatus?: boolean | null;
}

export default function LobbyDetails({
	lobby,
	players,
	countdown,
	lobbyState,
	sendMessage,
	userId,
	isKicking,
	onLeaveCheck,
	cachedPlayerConnectionStatus,
}: LobbyDetailsProps) {
	const [loading, setLoading] = useState<boolean>(false);
	const [connectionCheckLoading, setConnectionCheckLoading] =
		useState<boolean>(false);
	const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

	const handleLobbyState = async (state: lobbyState) => {
		setLoading(true);
		try {
			if (isKicking) {
				toast.info("Can't start game while kicking a player.");
				return;
			}
			await sendMessage({
				type: "updateLobbyState",
				newState: state,
			});
		} catch (error) {
			console.error("Failed to send message:", error);
			toast.error("Failed to update lobby state. Please try again.");
		} finally {
			setLoading(false);
		}
	};

	const checkConnectionStatus = async () => {
		if (!onLeaveCheck) return;

		setConnectionCheckLoading(true);
		try {
			onLeaveCheck(() => {
				setConnectionCheckLoading(false);
			});
		} catch (error) {
			console.error("Failed to check connection status:", error);
			setConnectionCheckLoading(false);
		}
	};

	// Reset connection check loading when connection status is received
	useEffect(() => {
		if (cachedPlayerConnectionStatus !== null && connectionCheckLoading) {
			setConnectionCheckLoading(false);
		}
	}, [cachedPlayerConnectionStatus, connectionCheckLoading]);

	const buttonLabel =
		lobbyState === "waiting"
			? "Start Game"
			: lobbyState === "starting" && countdown && countdown > 0
				? "Wait"
				: lobbyState === "finished"
					? "Ended"
					: "Loading...";

	const isDisabled =
		loading ||
		lobbyState === "finished" ||
		lobbyState === "inProgress" ||
		(lobbyState === "starting" && countdown === 0);

	const creator = players.find((p) => p.id === lobby.creator.id);
	const identifier = creator?.user.username || creator?.user.walletAddress;
	const isParticipant = players.some((p) => p.id === userId);

	return (
		<Card className="bg-primary/10 overflow-hidden">
			<CardHeader className="bg-muted/30 p-4 pb-3 sm:p-6 sm:pb-4">
				<CardTitle className="flex items-center gap-2 text-base sm:text-lg">
					<Info className="text-muted-foreground h-4 w-4 shrink-0 sm:h-5 sm:w-5" />
					<span className="truncate">Lobby Details</span>
				</CardTitle>
			</CardHeader>
			<CardContent className="p-4 sm:p-6">
				<div className="mt-3">
					<h3 className="text-muted-foreground mb-2 text-xs font-medium sm:mb-3 sm:text-sm">
						Created by
					</h3>
					<div className="bg-muted/30 hover:bg-muted/50 flex items-center justify-between rounded-lg p-2 transition-colors sm:p-3">
						<div className="flex min-w-0 flex-1 items-center gap-2 sm:gap-3">
							<div className="bg-primary/10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full sm:h-10 sm:w-10">
								<User className="text-primary h-4 w-4 sm:h-5 sm:w-5" />
							</div>
							<div className="min-w-0 flex-1">
								<Link
									href={`/u/${identifier}`}
									className="flex w-fit flex-col truncate"
								>
									<span className="truncate text-sm font-medium hover:underline sm:text-base">
										{creator?.user.displayName ||
											creator?.user.username ||
											truncateAddress(
												creator?.user.walletAddress
											) ||
											"Unknown Player"}
									</span>
									{(creator?.user.displayName ||
										creator?.user.username) && (
										<span className="text-muted-foreground truncate text-xs hover:underline">
											{truncateAddress(
												creator.user.walletAddress
											)}
										</span>
									)}
								</Link>
							</div>
						</div>
						{lobby.contractAddress && (
							<div className="ml-2 shrink-0">
								<Button
									variant={"link"}
									asChild
									size="sm"
									className="text-xs"
								>
									<Link
										href={`${EXPLORER_BASE_URL}txid/${lobby.contractAddress}?chain=${network}`}
										target="_blank"
										className="max-w-[100px] truncate sm:max-w-none"
									>
										<span className="hidden sm:inline">
											View Pool Contract
										</span>
										<span className="sm:hidden">
											View Contract
										</span>
									</Link>
								</Button>
							</div>
						)}
					</div>
				</div>

				{/* Countdown Timer */}
				{lobbyState === "starting" && (
					<div className="bg-muted/40 border-muted mt-6 rounded-md border p-4">
						<div className="flex items-center justify-center gap-2 text-center">
							<Timer className="text-muted-foreground h-5 w-5 shrink-0" />
							<span className="text-primary text-sm font-semibold sm:text-lg md:text-xl">
								Game starting in {countdown} seconds
							</span>
						</div>
					</div>
				)}

				{lobbyState === "inProgress" && (
					<div className="bg-muted/40 border-muted mt-6 space-y-3 rounded-md border p-4">
						<div className="flex items-center justify-center gap-2 text-center">
							<Info className="text-muted-foreground h-5 w-5 shrink-0" />
							<span className="text-primary text-sm font-semibold sm:text-lg md:text-xl">
								This game has already started.
							</span>
						</div>
						<div className="space-y-2 text-center">
							<span className="text-muted-foreground block text-xs sm:text-sm">
								You can spectate this ongoing game.
							</span>
							<Button
								asChild
								variant="default"
								size="sm"
								className="text-xs"
							>
								<Link href={`/lexi-wars/${lobby.id}`}>
									Spectate Game
								</Link>
							</Button>
						</div>
						{isParticipant &&
							lobby.entryAmount !== null &&
							lobby.entryAmount > 0 && (
								<div className="space-y-2 text-center">
									{cachedPlayerConnectionStatus === null ? (
										<div className="space-y-2">
											<span className="text-muted-foreground block text-xs sm:text-sm">
												Checking game state...
											</span>
											<Button
												variant="outline"
												size="sm"
												onClick={checkConnectionStatus}
												disabled={
													connectionCheckLoading
												}
												className="text-xs"
											>
												{connectionCheckLoading && (
													<Loader2 className="mr-1 h-3 w-3 animate-spin" />
												)}
												Check Status
											</Button>
										</div>
									) : (
										cachedPlayerConnectionStatus ===
											false && (
											<span className="text-muted-foreground text-xs sm:text-sm">
												You were unable to play, leave
												the lobby to withdraw your entry
												fee.
											</span>
										)
									)}
								</div>
							)}
					</div>
				)}

				{lobbyState === "finished" && (
					<div className="bg-destructive/10 border-destructive/20 mt-6 space-y-3 rounded-md border p-4">
						<div className="flex items-center justify-center gap-2 text-center">
							<Info className="text-destructive h-5 w-5 shrink-0" />
							<span className="text-destructive text-sm font-semibold sm:text-lg">
								This lobby has been closed
							</span>
						</div>
						<div className="space-y-2 text-center">
							<span className="text-muted-foreground block text-xs sm:text-sm">
								You can view the final results of this game.
							</span>
							<Button
								asChild
								variant="default"
								size="sm"
								className="text-xs"
							>
								<Link href={`/lexi-wars/${lobby.id}`}>
									View Results
								</Link>
							</Button>
						</div>
						{isParticipant &&
							lobby.entryAmount !== null &&
							lobby.entryAmount > 0 && (
								<div className="space-y-2 text-center">
									{cachedPlayerConnectionStatus === null ? (
										<div className="space-y-2">
											<span className="text-muted-foreground block text-xs sm:text-sm">
												Checking game state...
											</span>
											<Button
												variant="outline"
												size="sm"
												onClick={checkConnectionStatus}
												disabled={
													connectionCheckLoading
												}
												className="text-xs"
											>
												{connectionCheckLoading && (
													<Loader2 className="mr-1 h-3 w-3 animate-spin" />
												)}
												Check Status
											</Button>
										</div>
									) : (
										cachedPlayerConnectionStatus ===
											false && (
											<span className="text-muted-foreground text-xs sm:text-sm">
												You were unable to play, leave
												the lobby to withdraw your entry
												fee.
											</span>
										)
									)}
								</div>
							)}
					</div>
				)}

				{userId === lobby.creator.id && (
					<Button
						variant={
							lobbyState === "waiting" ? "default" : "destructive"
						}
						disabled={isDisabled}
						className="mt-6 w-full"
						onClick={() => {
							if (lobbyState === "waiting") {
								const now = Date.now();
								const activePlayers = players.filter(
									(player) => {
										const lastPingTime = player.lastPing;
										return lastPingTime
											? now - lastPingTime <= 30000
											: false;
									}
								);

								const totalPlayers = players.length;
								const activeCount = activePlayers.length;
								const requiredActive = Math.ceil(
									totalPlayers * 0.5
								);

								// Check if at least 2 active players
								if (activeCount < 2) {
									const inactiveCount =
										totalPlayers - activeCount;
									toast.info(
										"You need at least one more active player to start the game.",
										{
											description:
												inactiveCount > 0
													? `Consider removing ${inactiveCount} inactive player${inactiveCount > 1 ? "s" : ""} and inviting active ones.`
													: "Invite at least one more player to join the game.",
										}
									);
									return;
								}

								// Check if at least 50% of players are active
								if (activeCount < requiredActive) {
									const inactiveCount =
										totalPlayers - activeCount;
									toast.info(
										`Need ${requiredActive} active players out of ${totalPlayers} to start (50% minimum).`,
										{
											description: `Currently ${activeCount} active, ${inactiveCount} inactive. Remove inactive players or wait for them to reconnect.`,
										}
									);
									return;
								}

								handleLobbyState("starting");
							} else if (lobbyState === "starting")
								handleLobbyState("waiting");
						}}
					>
						{loading && (
							<Loader2 className="mr-2 h-4 w-4 animate-spin" />
						)}
						<span className="truncate">{buttonLabel}</span>
					</Button>
				)}
			</CardContent>
		</Card>
	);
}
