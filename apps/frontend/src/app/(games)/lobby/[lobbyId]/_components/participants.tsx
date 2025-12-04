import { Card, CardContent, CardTitle, CardHeader } from "@/components/ui/card";
import { Loader2, User as UserIcon, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { formatNumber, truncateAddress } from "@/lib/utils";
import { LobbyClientMessage } from "@/hooks/useLobbySocket";
import { useState } from "react";
import { EXPLORER_BASE_URL } from "@/lib/constants";
import { Lobby, PendingJoin } from "@/types/schema/lobby";
import {
	Player,
	//PlayerStatus
} from "@/types/schema/player";
import { kickFromFtPool, kickFromPool } from "@/lib/actions/kickPlayer";
import { waitForTxConfirmed } from "@/lib/actions/waitForTxConfirmed";
import { toast } from "sonner";

interface ParticipantProps {
	lobby: Lobby;
	players: Player[];
	pendingPlayers: PendingJoin[];
	userId: string;
	sendMessage: (msg: LobbyClientMessage) => Promise<void>;
	isKicking: boolean;
	setIsKicking: (kicking: boolean) => void;
}

export default function Participants({
	lobby,
	players,
	pendingPlayers,
	userId,
	sendMessage,
	isKicking,
	setIsKicking,
}: ParticipantProps) {
	//const currentPlayer = players.find((p) => p.id === userId);
	//const isReady = currentPlayer?.state === "ready";
	//const [isUpdating, setIsUpdating] = useState(false);
	const [isHandlingJoin, setIsHandlingJoin] = useState(false);
	const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

	const handleKickPlayer = async (
		playerId: string,
		playerAddress: string
	) => {
		setIsKicking(true);
		try {
			if (lobby.contractAddress && lobby.entryAmount !== null) {
				let kickTxId;
				if (lobby.tokenSymbol === "STX") {
					kickTxId = await kickFromPool(
						lobby.contractAddress as `${string}.${string}`,
						playerAddress,
						lobby.entryAmount
					);
				} else {
					if (!lobby.tokenId) {
						throw new Error("Token Id is missing");
					}
					kickTxId = await kickFromFtPool(
						lobby.contractAddress as `${string}.${string}`,
						lobby.tokenId,
						playerAddress,
						lobby.entryAmount
					);
				}

				if (!kickTxId) {
					throw new Error(
						"Failed to leave game pool: missing transaction ID"
					);
				}

				await waitForTxConfirmed(kickTxId);
			}
			await sendMessage({
				type: "kickPlayer",
				playerId,
			});
		} catch (error) {
			console.error("Error kicking player:", error);
			toast.error("Failed to kick player");
		} finally {
			setIsKicking(false);
		}
	};

	//const handleUpdatePlayerStatus = async (status: PlayerStatus) => {
	//	setIsUpdating(true);
	//	try {
	//		await sendMessage({
	//			type: "updatePlayerState",
	//			newState: status,
	//		});
	//	} catch (error) {
	//		console.error("Error updating status:", error);
	//	} finally {
	//		setIsUpdating(false);
	//	}
	//};

	const handleJoinRequest = async (userId: string, allow: boolean) => {
		setIsHandlingJoin(true);
		try {
			await sendMessage({
				type: "permitJoin",
				userId,
				allow,
			});
		} catch (error) {
			console.error("Error handling join request:", error);
			toast.error("Failed to handle join request");
		} finally {
			setIsHandlingJoin(false);
		}
	};

	return (
		<Card className="bg-primary/10 overflow-hidden">
			<CardHeader className="bg-muted/30 p-4 pb-3 sm:p-6 sm:pb-4">
				<div className="flex min-w-0 items-center justify-between">
					<CardTitle className="flex min-w-0 flex-1 items-center gap-2 text-base sm:text-lg">
						<Users className="text-muted-foreground h-4 w-4 shrink-0 sm:h-5 sm:w-5" />
						<span className="truncate">Current Participants</span>
					</CardTitle>
					{/*{userId !== lobby.creator.id &&
						!lobby.contractAddress &&
						players.some((p) => p.id === userId) && (
							<Button
								size="sm"
								variant={isReady ? "destructive" : "default"}
								disabled={isUpdating}
								onClick={() =>
									handleUpdatePlayerStatus(
										isReady ? "notReady" : "ready"
									)
								}
								className="shrink-0 ml-2"
							>
								{isUpdating && (
									<Loader2 className="h-4 w-4 mr-2 animate-spin" />
								)}
								{isReady ? "Unready" : "Ready"}
							</Button>
						)}*/}
				</div>
			</CardHeader>
			<CardContent className="p-4 sm:p-6">
				{players.length > 0 ? (
					<>
						<div className="space-y-2 sm:space-y-3">
							{players.map((player, index) => {
								const identifier =
									player.user.username ||
									player.user.walletAddress;
								const displayName =
									player.user.displayName ||
									player.user.username ||
									truncateAddress(player.user.walletAddress);
								const isCreator =
									player.id === lobby.creator.id;
								const isSelfCreator =
									userId === lobby.creator.id;
								const isSelf = userId === player.id;

								const now = Date.now();
								const lastPingTime = player.lastPing;
								const isActive = lastPingTime
									? now - lastPingTime <= 30000 // 30 seconds
									: false;

								return (
									<div
										key={index}
										className="bg-muted/30 hover:bg-muted/50 flex min-w-0 items-center justify-between rounded-lg p-2 transition-colors sm:p-3"
									>
										<div className="flex min-w-0 flex-1 items-center gap-2 sm:gap-3">
											<div className="bg-primary/10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full sm:h-10 sm:w-10">
												<UserIcon className="text-primary h-4 w-4 sm:h-5 sm:w-5" />
											</div>
											<div className="min-w-0 flex-1">
												<div className="flex flex-wrap items-center gap-2">
													<Link
														href={`/u/${identifier}`}
														className="flex flex-col truncate"
													>
														<span className="truncate text-sm font-medium hover:underline sm:text-base">
															{displayName}
														</span>
														{(player.user
															.displayName ||
															player.user
																.username) && (
															<span className="text-muted-foreground truncate text-xs hover:underline">
																{truncateAddress(
																	player.user
																		.walletAddress
																)}
															</span>
														)}
													</Link>
													<div className="flex shrink-0 flex-wrap items-center gap-1">
														{isCreator && (
															<span className="bg-primary/10 text-primary rounded-full px-2 py-0.5 text-xs whitespace-nowrap">
																Creator
															</span>
														)}
														{isSelf && (
															<span className="bg-accent/90 rounded-full px-2 py-0.5 text-xs whitespace-nowrap">
																You
															</span>
														)}
														{/*<span
															className={`text-xs px-2 py-0.5 rounded-full whitespace-nowrap ${
																player.state ===
																"ready"
																	? "bg-green-500/10 text-green-500"
																	: "bg-yellow-500/10 text-yellow-500"
															}`}
														>
															{player.state ===
															"ready"
																? "Ready"
																: "Not Ready"}
														</span>*/}
														<span
															className={`rounded-full px-2 py-0.5 text-xs whitespace-nowrap ${
																isActive
																	? "bg-emerald-500/10 text-emerald-500"
																	: "bg-red-500/10 text-red-500"
															}`}
														>
															{isActive
																? "Active"
																: "Inactive"}
														</span>
													</div>
												</div>
											</div>
										</div>
										<div className="ml-2 flex shrink-0 items-center gap-2">
											{player.txId &&
												lobby.entryAmount !== null && (
													<div className="flex flex-col items-end gap-1">
														<span className="text-sm font-bold whitespace-nowrap sm:text-base">
															{formatNumber(
																isCreator &&
																	lobby.entryAmount ===
																		0
																	? lobby.currentAmount ||
																			0
																	: lobby.entryAmount ||
																			0
															)}{" "}
															{lobby.tokenSymbol}
														</span>
														<Button
															variant={"link"}
															asChild
															className="h-auto p-0! text-right text-xs"
														>
															<Link
																href={`${EXPLORER_BASE_URL}txid/${player.txId}?chain=${network}`}
																target="_blank"
																className="max-w-[80px] truncate sm:max-w-none"
															>
																<span className="hidden sm:inline">
																	View in
																	explorer
																</span>
																<span className="sm:hidden">
																	Explorer
																</span>
															</Link>
														</Button>
													</div>
												)}
											{isSelfCreator && !isCreator && (
												<Button
													variant="destructive"
													size="sm"
													className="text-xs"
													disabled={isKicking}
													onClick={() =>
														handleKickPlayer(
															player.id,
															player.user
																.walletAddress
														)
													}
												>
													{isKicking && (
														<Loader2 className="mr-2 h-4 w-4 animate-spin" />
													)}
													Kick
												</Button>
											)}
										</div>
									</div>
								);
							})}
						</div>
						{pendingPlayers.length > 0 && (
							<>
								<h4 className="text-muted-foreground mt-6 mb-2 text-sm font-semibold sm:text-base">
									Pending Join Requests
								</h4>
								<div className="space-y-2 sm:space-y-3">
									{pendingPlayers.map((pendingplayer) => {
										const identifier =
											pendingplayer.user.username ||
											pendingplayer.user.walletAddress;
										const displayName =
											pendingplayer.user.displayName ||
											pendingplayer.user.username ||
											truncateAddress(
												pendingplayer.user.walletAddress
											);

										return (
											<div
												key={pendingplayer.user.id}
												className="bg-muted/20 hover:bg-muted/40 flex min-w-0 items-center justify-between rounded-lg p-2 transition-colors sm:p-3"
											>
												<div className="flex min-w-0 flex-1 items-center gap-2 sm:gap-3">
													<div className="bg-primary/10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full sm:h-10 sm:w-10">
														<UserIcon className="text-primary h-4 w-4 sm:h-5 sm:w-5" />
													</div>
													<div className="min-w-0 flex-1">
														<Link
															href={`/u/${identifier}`}
															className="flex flex-col truncate"
														>
															<span className="truncate text-sm font-medium hover:underline sm:text-base">
																{displayName}
															</span>
															{(pendingplayer.user
																.displayName ||
																pendingplayer
																	.user
																	.username) && (
																<span className="text-muted-foreground truncate text-xs hover:underline">
																	{truncateAddress(
																		pendingplayer
																			.user
																			.walletAddress
																	)}
																</span>
															)}
														</Link>
														<p className="text-muted-foreground text-xs">
															Requesting to join
														</p>
													</div>
												</div>
												{userId ===
													lobby.creator.id && (
													<div className="ml-2 flex shrink-0 gap-2">
														<Button
															size="sm"
															variant="outline"
															disabled={
																isHandlingJoin
															}
															onClick={() =>
																handleJoinRequest(
																	pendingplayer
																		.user
																		.id,
																	true
																)
															}
														>
															{isHandlingJoin && (
																<Loader2 className="mr-2 h-4 w-4 animate-spin" />
															)}
															Accept
														</Button>
														<Button
															size="sm"
															variant="destructive"
															disabled={
																isHandlingJoin
															}
															onClick={() =>
																handleJoinRequest(
																	pendingplayer
																		.user
																		.id,
																	false
																)
															}
														>
															{isHandlingJoin && (
																<Loader2 className="mr-2 h-4 w-4 animate-spin" />
															)}
															Decline
														</Button>
													</div>
												)}
											</div>
										);
									})}
								</div>
							</>
						)}
					</>
				) : (
					<div className="flex flex-col items-center justify-center py-6 text-center sm:py-8">
						<div className="bg-muted/50 mb-3 flex h-12 w-12 items-center justify-center rounded-full sm:mb-4 sm:h-16 sm:w-16">
							<Users className="text-muted-foreground h-6 w-6 sm:h-8 sm:w-8" />
						</div>
						<h3 className="mb-1 text-base font-medium sm:text-lg">
							No participants yet
						</h3>
						<p className="text-muted-foreground max-w-xs text-xs wrap-break-word sm:text-sm">
							Trust me something is wrong if you see this. Where
							da creator at?
						</p>
					</div>
				)}
			</CardContent>
		</Card>
	);
}
