"use client";
import GameHeader from "./game-header";
import GameRule from "./game-rule";
import GameTimer from "./game-timer";
import TurnIndicator from "./turn-indicator";
import LexiInputForm from "./lexi-input-form";
import GameOverModal from "./game-over-modal";
import { FormEvent, useCallback, useEffect, useState } from "react";
import {
	LexiWarsServerMessage,
	useLexiWarsSocket,
} from "@/hooks/useLexiWarsSocket";
import { toast } from "sonner";
import { truncateAddress } from "@/lib/utils";
import ClaimRewardModal from "./claim-reward-modal";
import ConnectionStatus from "@/components/connection-status";
import { useRouter } from "next/navigation";
import Loading from "../loading";
import Back from "./back";
import { useChatSocketContext } from "@/contexts/ChatSocketProvider";
import { Player, PlayerStanding } from "@/types/schema/player";
import { Lobby } from "@/types/schema/lobby";

interface LexiWarsProps {
	lobbyId: string;
	userId: string;
	lobby: Lobby;
}

export default function LexiWars({ lobbyId, userId, lobby }: LexiWarsProps) {
	const [word, setWord] = useState<string>("");

	const [turnState, setTurnState] = useState<{
		currentPlayer: Player | null;
		countdown: number | null;
	}>({
		currentPlayer: null,
		countdown: null,
	});
	const [rule, setRule] = useState<string>(
		"Word must be at least 4 characters!"
	);
	const [countdown, setCountdown] = useState<number>(15);
	const [rank, setRank] = useState<string | null>(null);
	const [finalStanding, setFinalStanding] = useState<PlayerStanding[] | null>(
		null
	);
	const [showPrizeModal, setShowPrizeModal] = useState(false);
	const [prizeAmount, setPrizeAmount] = useState<number | null>(null);
	const [isClaimed, setIsClaimed] = useState(false);
	const [latency, setLatency] = useState<number | null>(null);
	const [isLoading, setIsLoading] = useState<boolean>(false);
	const [gameStarted, setGameStarted] = useState<boolean>(false);
	const [startCountdown, setStartCountdown] = useState<number>(15);
	const [gameOver, setGameOver] = useState<boolean>(false);
	const [isSpectator, setIsSpectator] = useState<boolean>(false);
	const [warsPoint, setWarsPoint] = useState<number | null>(null);
	const [startFailed, setStartFailed] = useState<boolean>(false);
	const [playersCount, setPlayersCount] = useState<{
		connectedPlayers: number;
		remainingPlayers: number;
	} | null>(null);

	const router = useRouter();

	const handleMessage = useCallback(
		(message: LexiWarsServerMessage) => {
			switch (message.type) {
				case "turn":
					setTurnState({
						currentPlayer: message.currentTurn,
						countdown: message.countdown,
					});
					break;
				case "rule":
					setRule(message.rule);
					break;
				case "countdown":
					setCountdown(message.time);
					break;
				case "rank":
					setRank(message.rank);
					toast.info(`Time's up!`, {
						description: `Your rank was ${message.rank}.`,
					});
					break;
				case "validate":
					toast.info(`${message.msg}`);
					break;
				case "wordEntry":
					toast.info(
						`${
							userId === message.sender.id
								? "You"
								: message.sender.user.displayName ||
									message.sender.user.username ||
									truncateAddress(
										message.sender.user.walletAddress
									)
						} entered: ${message.word}`
					);
					break;
				case "usedWord":
					toast.info(`${message.word} has already been used!`);
					break;
				case "gameOver":
					toast.info(`🏁 Game Over!`);
					setGameOver(true);
					break;
				case "finalStanding":
					setFinalStanding(message.standing);
					break;
				case "prize":
					setPrizeAmount(message.amount);
					if (message.amount > 0) {
						setShowPrizeModal(true);
						setIsClaimed(false);
					} else if (message.amount === 0) {
						setIsClaimed(true);
					}
					break;
				case "warsPoint":
					setWarsPoint(message.warsPoint);
					setShowPrizeModal(true);
					if (!lobby.contractAddress) {
						setPrizeAmount(0);
					}
					break;
				case "pong":
					setLatency(message.pong);
					break;
				case "start":
					setStartCountdown(message.time);
					setGameStarted(message.started);
					break;
				case "startFailed":
					setStartFailed(true);
					break;
				case "spectator":
					setIsSpectator(true);
					setCountdown(15);
					console.log("Ya spectating");
					toast.info("You are spectating this game");
					break;
				case "playersCount":
					setPlayersCount({
						connectedPlayers: message.connectedPlayers,
						remainingPlayers: message.remainingPlayers,
					});
					break;
				default:
					console.warn("Unknown WS message type", message);
			}
		},
		[userId, lobby.contractAddress]
	);

	const {
		sendMessage,
		readyState,
		reconnecting,
		forceReconnect,
		disconnect,
	} = useLexiWarsSocket({
		lobbyId,
		userId,
		onMessage: handleMessage,
	});

	useEffect(() => {
		if (startFailed) {
			toast.error("Failed to start the game.", {
				description: "Not enough players connected.",
			});
			disconnect();
			router.replace(`/lobby/${lobbyId}`);
		}
	}, [startFailed, disconnect, lobbyId, router]);

	const { disconnectChat } = useChatSocketContext();

	useEffect(() => {
		const shouldDisconnect =
			finalStanding &&
			gameOver &&
			(lobby.contractAddress ? prizeAmount !== null : true);

		if (shouldDisconnect) {
			disconnect();
			disconnectChat();
		}
	}, [
		finalStanding,
		gameOver,
		prizeAmount,
		disconnect,
		disconnectChat,
		lobby.contractAddress,
	]);

	const handleSubmit = async (e?: FormEvent) => {
		setIsLoading(true);
		try {
			e?.preventDefault();
			await sendMessage({ type: "wordEntry", word: word.trim() });
			setWord("");
		} catch (error) {
			console.error("Failed to send word:", error);
			toast.error("Failed to send word. Please try again.");
		} finally {
			setIsLoading(false);
		}
	};

	if (!gameStarted && lobby.state !== "finished") {
		return (
			<Loading
				startCountdown={startCountdown}
				readyState={readyState}
				reconnecting={reconnecting}
				latency={latency}
				onForceReconnect={forceReconnect}
			/>
		);
	}

	return (
		<main className="min-h-screen bg-gradient-to-b from-background to-primary/30">
			<div className="max-w-3xl mx-auto p-4 sm:p-6 ">
				<div className="flex justify-between">
					<Back
						isOut={
							rank !== null || finalStanding !== null
								? true
								: false
						}
						isSpectator={isSpectator}
						disconnect={disconnect}
						disconnectChat={disconnectChat}
					/>
					<ConnectionStatus
						className="mb-4 sm:mb-6"
						readyState={readyState}
						latency={latency}
						reconnecting={reconnecting}
						onReconnect={forceReconnect}
					/>
				</div>
				<div className="space-y-3 sm:space-y-4">
					<GameHeader />
					<GameTimer timeLeft={countdown} />
					{((turnState.currentPlayer &&
						turnState.currentPlayer.id === userId) ||
						isSpectator) && <GameRule currentRule={rule} />}
					<div className="border border-primary/10 p-3 sm:p-4 bg-primary/10 rounded-xl shadow-sm space-y-4 sm:space-y-5">
						<TurnIndicator
							currentPlayer={turnState.currentPlayer}
							userId={userId}
							countdown={turnState.countdown}
						/>
						<LexiInputForm
							word={word}
							setWord={setWord}
							handleSubmit={handleSubmit}
							isLoading={isLoading}
							disabled={isSpectator}
						/>
					</div>
				</div>

				{playersCount && (
					<div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 z-10 max-w-3xl w-full px-4">
						<div className="flex items-center justify-around gap-4 p-3 bg-primary/10 border border-primary/30 rounded-lg shadow-lg">
							<span className="text-sm font-medium text-muted-foreground">
								Total Players:{" "}
								<span className="text-foreground font-bold">
									{playersCount.connectedPlayers}
								</span>
							</span>
							<span className="text-sm font-medium text-muted-foreground">
								Still Playing:{" "}
								<span className="text-foreground font-bold">
									{playersCount.remainingPlayers}
								</span>
							</span>
						</div>
					</div>
				)}

				{showPrizeModal && (
					<ClaimRewardModal
						showPrizeModal={showPrizeModal}
						setShowPrizeModal={setShowPrizeModal}
						setIsClaimed={setIsClaimed}
						rank={rank}
						prizeAmount={prizeAmount}
						lobbyId={lobbyId}
						warsPoint={warsPoint}
						lobby={lobby}
					/>
				)}

				{finalStanding && !showPrizeModal && (
					<GameOverModal
						standing={finalStanding}
						userId={userId}
						contractAddress={lobby.contractAddress}
						isClaimed={isClaimed}
						creatorId={lobby.creator.id}
					/>
				)}

				<div className="sr-only" role="alert">
					This is a competitive typing game that requires manual
					keyboard input. Screen readers softwares are not supported
					for fair gameplay.
				</div>
			</div>
		</main>
	);
}
