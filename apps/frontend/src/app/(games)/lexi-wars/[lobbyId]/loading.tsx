import Image from "next/image";
import ConnectionStatus from "@/components/connection-status";

interface LoadingProps {
	startCountdown?: number;
	readyState?: number;
	reconnecting?: boolean;
	latency?: number | null;
	onForceReconnect?: () => void;
}

export default function Loading({
	startCountdown,
	readyState,
	reconnecting,
	latency,
	onForceReconnect,
}: LoadingProps) {
	return (
		<main className="min-h-screen bg-gradient-to-b from-background to-primary/30">
			<div className="max-w-3xl mx-auto p-4 sm:p-6">
				<div className="min-h-[70vh] flex flex-col items-center justify-center space-y-8">
					<div className="animate-bounce">
						<Image
							src="/logos/lexi-wars.webp"
							alt="Lexi Wars"
							width={300}
							height={300}
							className="w-48 h-48 sm:w-64 sm:h-64 md:w-72 md:h-72"
							//priority
						/>
					</div>

					{/* Loading Content */}
					<div className="text-center space-y-4">
						<div className="space-y-2">
							<h2 className="text-2xl sm:text-3xl font-bold text-foreground">
								Preparing Battle Arena
							</h2>
							{startCountdown !== undefined && (
								<p className="text-lg sm:text-xl text-muted-foreground">
									Game starting in{" "}
									<span className="font-semibold text-primary">
										{startCountdown}
									</span>{" "}
									seconds
								</p>
							)}
						</div>

						{/* Connection Status */}
						{readyState !== undefined && (
							<div className="flex justify-center">
								<ConnectionStatus
									readyState={readyState}
									latency={latency ?? null}
									reconnecting={reconnecting ?? false}
									onReconnect={onForceReconnect}
									className="px-3 py-2 rounded-lg bg-background/50 border border-primary/20"
								/>
							</div>
						)}

						{/* Game Tip */}
						<div className="mt-8 p-4 bg-primary/10 rounded-lg border border-primary/20 max-w-md mx-auto">
							{readyState !== undefined &&
							readyState !== WebSocket.OPEN ? (
								<p className="text-sm text-muted-foreground">
									<span className="font-semibold text-yellow-400">
										⚠️ Connection Issue:
									</span>
									<br />
									If you&apos;re stuck here, try the retry
									button above or refresh the page.
								</p>
							) : (
								<p className="text-sm text-muted-foreground">
									<span className="font-semibold text-primary">
										💡 Game Tip:
									</span>
									<br />
									Always look at the turn indicator to know
									when it&apos;s your turn
								</p>
							)}
						</div>
					</div>

					<div className="flex space-x-2">
						<div className="w-2 h-2 bg-primary rounded-full animate-pulse"></div>
						<div className="w-2 h-2 bg-primary rounded-full animate-pulse delay-75"></div>
						<div className="w-2 h-2 bg-primary rounded-full animate-pulse delay-150"></div>
					</div>
				</div>
			</div>
		</main>
	);
}
