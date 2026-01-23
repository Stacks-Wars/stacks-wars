import type { GamePluginProps } from "@/lib/definitions";
import type { CoinFlipState } from "./types";

export default function CoinFlipGame({
	state,
	sendMessage,
	lobby,
}: GamePluginProps<CoinFlipState>) {
	const handleGuess = (guess: "heads" | "tails") => {
		sendMessage("make_guess", { guess });
	};

	const isWaitingForGuesses = state.currentRound > 0 && !state.lastCoinResult;
	const hasGuessed = state.currentPlayer
		? Object.keys(state.guesses).includes(state.currentPlayer)
		: false;

	return (
		<div className="container mx-auto px-4 py-8">
			<div className="space-y-6">
				{/* Game Header */}
				<div className="rounded-lg border bg-card p-6">
					<h2 className="text-2xl font-bold">{lobby.name}</h2>
					<p className="text-muted-foreground">Coin Flip</p>
					{state.currentRound > 0 && (
						<p className="mt-2 text-sm text-muted-foreground">
							Round {state.currentRound}
						</p>
					)}
				</div>

				{/* Coin Display */}
				<div className="flex items-center justify-center p-12">
					<div
						className={`flex h-32 w-32 items-center justify-center rounded-full border-4 text-4xl font-bold ${
							isWaitingForGuesses
								? "border-yellow-500 bg-yellow-500/20"
								: state.lastCoinResult
									? "border-primary bg-primary/20"
									: "border-muted bg-muted"
						}`}
					>
						{state.lastCoinResult
							? state.lastCoinResult[0].toUpperCase()
							: "?"}
					</div>
				</div>

				{/* Game Status */}
				{state.currentPlayer && (
					<div className="rounded-lg border bg-card p-4 text-center">
						<p className="text-sm text-muted-foreground">
							Current Player
						</p>
						<p className="font-mono font-semibold">
							{state.currentPlayer.slice(0, 8)}...
						</p>
					</div>
				)}

				{/* Guess Buttons */}
				{isWaitingForGuesses && !hasGuessed && (
					<div className="grid grid-cols-2 gap-4">
						<button
							onClick={() => handleGuess("heads")}
							className="rounded-lg border-2 border-primary bg-primary/10 p-6 text-xl font-semibold hover:bg-primary/20"
						>
							🪙 HEADS
						</button>
						<button
							onClick={() => handleGuess("tails")}
							className="rounded-lg border-2 border-primary bg-primary/10 p-6 text-xl font-semibold hover:bg-primary/20"
						>
							🪙 TAILS
						</button>
					</div>
				)}

				{/* Waiting Message */}
				{hasGuessed && isWaitingForGuesses && (
					<div className="rounded-lg border bg-card p-6 text-center">
						<p className="text-muted-foreground">
							Waiting for other players...
						</p>
					</div>
				)}

				{/* Players Status */}
				<div className="rounded-lg border bg-card p-6">
					<h3 className="mb-4 font-semibold">Players</h3>
					<div className="space-y-2">
						{state.players.map((userId) => {
							const isActive =
								state.activePlayers.includes(userId);
							const isEliminated =
								state.eliminatedPlayers.includes(userId);
							const hasGuessed = Object.keys(
								state.guesses
							).includes(userId);
							const guess = state.guesses[userId];

							return (
								<div
									key={userId}
									className={`flex items-center justify-between rounded p-2 ${
										isEliminated
											? "line-through opacity-50"
											: ""
									}`}
								>
									<span className="font-mono text-sm">
										{userId.slice(0, 8)}...
									</span>
									<div className="flex items-center gap-2">
										{guess && (
											<span className="rounded bg-muted px-2 py-1 text-xs">
												{guess.toUpperCase()}
											</span>
										)}
										{hasGuessed && isWaitingForGuesses && (
											<span className="text-xs text-green-600">
												✓
											</span>
										)}
										{isEliminated && (
											<span className="text-xs text-red-600">
												❌
											</span>
										)}
										{isActive && (
											<span className="text-xs text-primary">
												Active
											</span>
										)}
									</div>
								</div>
							);
						})}
					</div>
				</div>

				{/* Round Result */}
				{state.lastCoinResult && (
					<div className="rounded-lg border bg-card p-6 text-center">
						<h3 className="mb-2 text-xl font-semibold">
							Coin Result: {state.lastCoinResult.toUpperCase()}!
						</h3>
						<p className="text-sm text-muted-foreground">
							{state.activePlayers.length} players remaining
						</p>
					</div>
				)}

				{/* Game Finished */}
				{state.finished && state.results && (
					<div className="rounded-lg border bg-card p-6">
						<h3 className="mb-4 text-center text-xl font-semibold">
							Game Over!
						</h3>
						<div className="space-y-2">
							{state.results.rankings.map((ranking) => (
								<div
									key={ranking.userId}
									className="flex items-center justify-between rounded bg-muted p-2"
								>
									<div className="flex items-center gap-3">
										<span className="text-2xl font-bold">
											#{ranking.rank}
										</span>
										<span className="font-mono text-sm">
											{ranking.userId.slice(0, 8)}...
										</span>
									</div>
									{ranking.rank === 1 && (
										<span className="text-2xl">🏆</span>
									)}
								</div>
							))}
						</div>
					</div>
				)}
			</div>
		</div>
	);
}
