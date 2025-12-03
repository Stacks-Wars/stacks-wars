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
		<main className="from-background to-primary/30 min-h-screen bg-gradient-to-b">
			<div className="mx-auto max-w-3xl p-4 sm:p-6">
				<div className="flex min-h-[70vh] flex-col items-center justify-center space-y-8">
					<div className="animate-bounce">
						<Image
							src="/logos/lexi-wars.webp"
							alt="Lexi Wars"
							width={300}
							height={300}
							className="h-48 w-48 sm:h-64 sm:w-64 md:h-72 md:w-72"
							//priority
						/>
					</div>

					{/* Loading Content */}
					<div className="space-y-4 text-center">
						<div className="space-y-2">
							<h2 className="text-foreground text-2xl font-bold sm:text-3xl">
								Preparing Battle Arena
							</h2>
							{startCountdown !== undefined && (
								<p className="text-muted-foreground text-lg sm:text-xl">
									Game starting in{" "}
									<span className="text-primary font-semibold">
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
									className="bg-background/50 border-primary/20 rounded-lg border px-3 py-2"
								/>
							</div>
						)}

						{/* Game Tip */}
						<div className="bg-primary/10 border-primary/20 mx-auto mt-8 max-w-md rounded-lg border p-4">
							{readyState !== undefined &&
							readyState !== WebSocket.OPEN ? (
								<p className="text-muted-foreground text-sm">
									<span className="font-semibold text-yellow-400">
										⚠️ Connection Issue:
									</span>
									<br />
									If you&apos;re stuck here, try the retry
									button above or refresh the page.
								</p>
							) : (
								<p className="text-muted-foreground text-sm">
									<span className="text-primary font-semibold">
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
						<div className="bg-primary h-2 w-2 animate-pulse rounded-full"></div>
						<div className="bg-primary h-2 w-2 animate-pulse rounded-full delay-75"></div>
						<div className="bg-primary h-2 w-2 animate-pulse rounded-full delay-150"></div>
					</div>
				</div>
			</div>
		</main>
	);
}
