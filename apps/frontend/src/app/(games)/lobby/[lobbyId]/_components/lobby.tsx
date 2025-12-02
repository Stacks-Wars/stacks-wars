"use client";
import { ArrowLeft, Loader } from "lucide-react";
import LobbyDetails from "./lobby-details";
import LobbyStats from "./lobby-stats";
import Participants from "./participants";
import JoinLobbyForm from "./join-lobby-form";
import GamePreview from "./game-preview";
import { Suspense, useCallback, useEffect, useState } from "react";
import { LobbyServerMessage, useLobbySocket } from "@/hooks/useLobbySocket";
import { toast } from "sonner";
import { truncateAddress } from "@/lib/utils";
import { useRouter } from "next/navigation";
import ConnectionStatus from "@/components/connection-status";
import ShareLinkButton from "./share-link-button";
import Link from "next/link";
//import Loading from "@/app/(games)/lexi-wars/[lobbyId]/loading";
import { useChatSocketContext } from "@/contexts/ChatSocketProvider";
import { GameType } from "@/types/schema/game";
import {
	JoinState,
	lobbyState,
	Lobby as LobbyType,
	PendingJoin,
} from "@/types/schema/lobby";
import { Player } from "@/types/schema/player";

interface LobbyProps {
	lobby: LobbyType;
	players: Player[];
	userId: string;
	lobbyId: string;
	game: GameType;
}
export default function Lobby({
	lobby,
	players,
	userId,
	lobbyId,
	game,
}: LobbyProps) {
	const [joined, setJoined] = useState(false);
	const [participantList, setParticipantList] = useState<Player[]>(players);
	const [countdown, setCountdown] = useState<number | null>(null);
	const [lobbyState, setLobbyState] = useState<lobbyState>(lobby.state);
	const [pendingPlayers, setPendingPlayers] = useState<PendingJoin[]>([]);
	const [joinState, setJoinState] = useState<JoinState | null>(null);
	const [latency, setLatency] = useState<number | null>(null);
	const [prefetched, setPrefetched] = useState(false);
	const [isKicking, setIsKicking] = useState(false);
	const [started, setStarted] = useState(false);
	const [leaveCheckCallback, setLeaveCheckCallback] = useState<
		((isConnected: boolean) => void) | null
	>(null);
	const [cachedPlayerConnectionStatus, setCachedPlayerConnectionStatus] =
		useState<boolean | null>(null);

	const router = useRouter();

	const isParticipant = participantList.some((p) => p.id === userId);

	const handleMessage = useCallback(
		(message: LobbyServerMessage) => {
			switch (message.type) {
				case "playerUpdated":
					setParticipantList(message.players);
					break;
				case "playerKicked":
					toast.info(
						`${
							message.player.displayName ||
							message.player.username ||
							truncateAddress(message.player.walletAddress)
						} was kicked from the lobby.`
					);
					break;
				case "notifyKicked":
					toast.info("You were kicked from the lobby.");
					window.location.reload();
					break;
				case "left":
					toast.info("You left the lobby.");
					window.location.reload();
					break;
				case "countdown":
					setCountdown(message.time);
					break;
				case "lobbyState":
					setLobbyState(message.state);
					setStarted(message.started);
					break;
				case "pendingPlayers":
					setPendingPlayers(
						message.pendingPlayers.filter(
							(p) => p.state === "pending"
						)
					);
					const isInPending = message.pendingPlayers.find(
						(p) => p.user.id === userId
					);
					if (isInPending) {
						setJoinState(isInPending.state);
					}
					break;
				case "playersNotJoined":
					const notJoinedPlayers = message.players;
					notJoinedPlayers.forEach((p) => {
						toast.error(
							`${p.user.displayName || p.user.username || truncateAddress(p.user.walletAddress)} is not ready`
						);
					});
					break;
				case "allowed":
					setJoinState(message.type);
					if (!isParticipant) toast.success("Join request approved!");
					break;
				case "rejected":
					setJoinState(message.type);
					toast.info("Join request was dropped");
					break;
				case "pending":
					setJoinState(message.type);
					toast.info("Your join request is pending approval");
					break;
				case "warsPointDeduction":
					toast.warning(`You lost ${message.amount} Wars Points`);
					break;
				case "error":
					toast.error(`Error: ${message.message}`);
					break;
				case "pong":
					setLatency(message.pong);
					break;
				case "isConnectedPlayer":
					setCachedPlayerConnectionStatus(message.response);
					if (leaveCheckCallback) {
						leaveCheckCallback(message.response);
						setLeaveCheckCallback(null);
					}
					break;
				default:
					console.warn("Unknown WS message type", message);
			}
		},
		[
			userId,
			isParticipant,
			leaveCheckCallback,
			setCachedPlayerConnectionStatus,
		]
	);

	const {
		sendMessage,
		readyState,
		reconnecting,
		forceReconnect,
		disconnect,
	} = useLobbySocket({
		lobbyId,
		userId,
		onMessage: handleMessage,
	});

	const { disconnectChat } = useChatSocketContext();

	const handleLeaveCheck = useCallback(
		(callback: (isConnected: boolean) => void) => {
			if (cachedPlayerConnectionStatus !== null) {
				callback(cachedPlayerConnectionStatus);
				return;
			}

			setLeaveCheckCallback(() => callback);
		},
		[cachedPlayerConnectionStatus]
	);

	useEffect(() => {
		if (isParticipant && !joined) {
			setJoined(true);
		} else if (!isParticipant && joined) {
			setJoined(false);
		}
	}, [isParticipant, joined]);

	useEffect(() => {
		if (countdown !== null && countdown < 15 && !prefetched) {
			router.prefetch(`/lexi-wars/${lobbyId}`);
			setPrefetched(true);
		}
	}, [countdown, lobbyId, prefetched, router]);

	useEffect(() => {
		if (lobbyState === "inProgress" && !started) {
			disconnect();
			if (isParticipant) {
				router.push(`/lexi-wars/${lobbyId}`);
			} else {
				//router.replace(`/lobby`);
			}
		} else if (lobbyState === "waiting") {
			//setCountdown(15);
		} else if (lobbyState === "finished") {
			if (isParticipant) {
				// Check connection status for participants before disconnecting
				handleLeaveCheck((isConnected: boolean) => {
					if (isConnected) {
						// Player was connected, they can't leave - disconnect both
						disconnect();
						disconnectChat();
					} else {
						// Player wasn't connected, they can leave - only disconnect chat
						disconnectChat();
					}
				});
			} else {
				// Non-participants can always disconnect both
				disconnect();
				disconnectChat();
			}
		}
	}, [
		started,
		lobbyState,
		isParticipant,
		router,
		lobbyId,
		disconnect,
		disconnectChat,
		handleLeaveCheck,
	]);

	return (
		<section className="bg-gradient-to-b from-primary/10 to-primary/30">
			<div className="max-w-7xl mx-auto px-4 py-4 sm:px-6 sm:py-6 ">
				<div className="flex items-center justify-between ">
					<Link
						href="/lobby"
						onClick={() => {
							disconnect();
							disconnectChat();
						}}
						className="inline-flex items-center gap-1 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
					>
						<ArrowLeft className="h-4 w-4" />
						<span>Back to Lobby</span>
					</Link>
					<ShareLinkButton lobbyId={lobbyId} />
				</div>
				<ConnectionStatus
					readyState={readyState}
					latency={latency}
					reconnecting={reconnecting}
					onReconnect={forceReconnect}
				/>
				<div className="grid gap-4 sm:gap-6 lg:gap-8 lg:grid-cols-3 mt-4 sm:mt-6">
					{/* Main Content */}
					<div className="lg:col-span-2 space-y-4 sm:space-y-6 lg:space-y-8">
						{/* Stats Cards */}
						<LobbyStats lobby={lobby} players={participantList} />
						{/* Lobby Details */}
						<LobbyDetails
							lobby={lobby}
							players={participantList}
							countdown={countdown}
							lobbyState={lobbyState}
							sendMessage={sendMessage}
							userId={userId}
							isKicking={isKicking}
							onLeaveCheck={handleLeaveCheck}
							cachedPlayerConnectionStatus={
								cachedPlayerConnectionStatus
							}
						/>
						<Participants
							lobby={lobby}
							players={participantList}
							pendingPlayers={pendingPlayers}
							userId={userId}
							sendMessage={sendMessage}
							isKicking={isKicking}
							setIsKicking={setIsKicking}
						/>
					</div>
					<div className="space-y-4 sm:space-y-6">
						<div className="lg:sticky lg:top-6 flex flex-col gap-4">
							<Suspense
								fallback={
									<div className="flex justify-center items-center py-6 sm:py-8">
										<Loader className="h-6 w-6 sm:h-8 sm:w-8 animate-spin text-muted-foreground" />
									</div>
								}
							>
								<JoinLobbyForm
									lobby={lobby}
									players={participantList}
									joinState={joinState}
									lobbyId={lobbyId}
									userId={userId}
									sendMessage={sendMessage}
									disconnect={disconnect}
									chatDisconnect={disconnectChat}
									onLeaveCheck={handleLeaveCheck}
									cachedPlayerConnectionStatus={
										cachedPlayerConnectionStatus
									}
								/>
							</Suspense>
							<GamePreview game={game} />
						</div>
					</div>
				</div>
			</div>
		</section>
	);
}
