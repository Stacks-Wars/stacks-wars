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
		<Card className="overflow-hidden bg-primary/10">
			<CardHeader className="bg-muted/30 p-4 pb-3 sm:p-6 sm:pb-4">
				<CardTitle className="flex items-center gap-2 text-base sm:text-lg">
					<Info className="h-4 w-4 sm:h-5 sm:w-5 text-muted-foreground shrink-0" />
					<span className="truncate">Lobby Details</span>
				</CardTitle>
			</CardHeader>
			<CardContent className="p-4 sm:p-6">
				<div className="mt-3">
					<h3 className="text-xs sm:text-sm font-medium text-muted-foreground mb-2 sm:mb-3">
						Created by
					</h3>
					<div className="flex justify-between items-center p-2 sm:p-3 rounded-lg bg-muted/30 hover:bg-muted/50 transition-colors">
						<div className="flex items-center gap-2 sm:gap-3 min-w-0 flex-1">
							<div className="h-8 w-8 sm:h-10 sm:w-10 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
								<User className="h-4 w-4 sm:h-5 sm:w-5 text-primary" />
							</div>
							<div className="min-w-0 flex-1">
								<Link
									href={`/u/${identifier}`}
									className="flex flex-col truncate w-fit"
								>
									<span className="text-sm sm:text-base font-medium truncate hover:underline">
										{creator?.user.displayName ||
											creator?.user.username ||
											truncateAddress(
												creator?.user.walletAddress
											) ||
											"Unknown Player"}
									</span>
									{(creator?.user.displayName ||
										creator?.user.username) && (
										<span className="text-xs text-muted-foreground truncate hover:underline">
											{truncateAddress(
												creator.user.walletAddress
											)}
										</span>
									)}
								</Link>
							</div>
						</div>
						{lobby.contractAddress && (
							<div className="shrink-0 ml-2">
								<Button
									variant={"link"}
									asChild
									size="sm"
									className="text-xs"
								>
									<Link
										href={`${EXPLORER_BASE_URL}txid/${lobby.contractAddress}?chain=${network}`}
										target="_blank"
										className="truncate max-w-[100px] sm:max-w-none"
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
					<div className="mt-6 p-4 rounded-md bg-muted/40 border border-muted">
						<div className="flex items-center justify-center gap-2 text-center">
							<Timer className="h-5 w-5 text-muted-foreground shrink-0" />
							<span className="text-sm sm:text-lg md:text-xl font-semibold text-primary">
								Game starting in {countdown} seconds
							</span>
						</div>
					</div>
				)}

				{lobbyState === "inProgress" && (
					<div className="mt-6 p-4 rounded-md bg-muted/40 border border-muted space-y-3">
						<div className="flex items-center justify-center gap-2 text-center">
							<Info className="h-5 w-5 text-muted-foreground shrink-0" />
							<span className="text-sm sm:text-lg md:text-xl font-semibold text-primary">
								This game has already started.
							</span>
						</div>
						<div className="text-center space-y-2">
							<span className="text-xs sm:text-sm text-muted-foreground block">
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
								<div className="text-center space-y-2">
									{cachedPlayerConnectionStatus === null ? (
										<div className="space-y-2">
											<span className="text-xs sm:text-sm text-muted-foreground block">
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
													<Loader2 className="h-3 w-3 mr-1 animate-spin" />
												)}
												Check Status
											</Button>
										</div>
									) : (
										cachedPlayerConnectionStatus ===
											false && (
											<span className="text-xs sm:text-sm text-muted-foreground">
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
					<div className="mt-6 p-4 rounded-md bg-destructive/10 border border-destructive/20 space-y-3">
						<div className="flex items-center justify-center gap-2 text-center">
							<Info className="h-5 w-5 text-destructive shrink-0" />
							<span className="text-sm sm:text-lg font-semibold text-destructive">
								This lobby has been closed
							</span>
						</div>
						<div className="text-center space-y-2">
							<span className="text-xs sm:text-sm text-muted-foreground block">
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
								<div className="text-center space-y-2">
									{cachedPlayerConnectionStatus === null ? (
										<div className="space-y-2">
											<span className="text-xs sm:text-sm text-muted-foreground block">
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
													<Loader2 className="h-3 w-3 mr-1 animate-spin" />
												)}
												Check Status
											</Button>
										</div>
									) : (
										cachedPlayerConnectionStatus ===
											false && (
											<span className="text-xs sm:text-sm text-muted-foreground">
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
						className="w-full mt-6"
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
							<Loader2 className="h-4 w-4 mr-2 animate-spin" />
						)}
						<span className="truncate">{buttonLabel}</span>
					</Button>
				)}
			</CardContent>
		</Card>
	);
}
