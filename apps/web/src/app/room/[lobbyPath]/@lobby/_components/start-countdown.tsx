import { Button } from "@/components/ui/button";
import { useCountdown, useIsActionLoading } from "@/lib/stores/room";

interface StartCountdownProps {
	isCreator: boolean;
	handleCancelStart: () => void;
}

export default function StartCountdown({
	isCreator,
	handleCancelStart,
}: StartCountdownProps) {
	const countdown = useCountdown();
	const isCancelGameLoading = useIsActionLoading("updateLobbyStatus-waiting");

	return (
		<>
			{countdown !== null && countdown > 0 && (
				<div className="bg-background/80 fixed inset-0 z-50 flex items-center justify-center backdrop-blur-sm">
					<div className="flex flex-col items-center gap-6 text-center">
						<p className="text-muted-foreground text-lg sm:text-xl">
							Game starting in
						</p>
						<div className="relative flex items-center justify-center">
							<div className="border-primary/20 absolute size-32 rounded-full border-4 sm:size-40 lg:size-48" />
							<div
								className="border-primary absolute size-32 animate-spin rounded-full border-4 border-t-transparent sm:size-40 lg:size-48"
								style={{ animationDuration: "1s" }}
							/>
							<span className="text-primary text-6xl font-bold sm:text-7xl lg:text-8xl">
								{countdown}
							</span>
						</div>
						{isCreator && (
							<Button
								variant="outline"
								size="lg"
								onClick={handleCancelStart}
								disabled={isCancelGameLoading}
								className="mt-4"
							>
								{isCancelGameLoading
									? "Cancelling..."
									: "Cancel"}
							</Button>
						)}
					</div>
				</div>
			)}
		</>
	);
}
