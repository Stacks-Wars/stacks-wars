import { useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { ChevronLeft, LogOut, Wifi, WifiOff } from "lucide-react";
import { useRoom } from "@/lib/contexts/room-context";
import { useRoomView } from "@/lib/contexts/room-view-context";
import ShareButton from "../../app/room/[lobbyPath]/@lobby/_components/share-button";
import { cn } from "@/lib/utils";
import {
	useRoomConnected,
	useRoomConnecting,
	useRoomLatency,
} from "@/lib/stores/room";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";

function getLatencyColor(latency: number): string {
	if (latency < 50) return "text-green-500"; // Very good
	if (latency < 100) return "text-lime-500"; // Good
	if (latency < 200) return "text-orange-500"; // Bad
	return "text-red-500"; // Very bad
}

function getLatencyLabel(latency: number): string {
	if (latency < 50) return "Excellent";
	if (latency < 100) return "Good";
	if (latency < 200) return "Poor";
	return "Bad";
}

interface RoomHeaderProps {
	slot?: "lobby" | "game";
}

export default function RoomHeader({ slot = "game" }: RoomHeaderProps) {
	const { disconnect, sendGameMessage } = useRoom();
	const latency = useRoomLatency();
	const isConnected = useRoomConnected();
	const isConnecting = useRoomConnecting();
	const [showQuitDialog, setShowQuitDialog] = useState(false);

	const handleBackClick = () => {
		// Disconnect WebSocket before navigating away
		disconnect();
	};

	const handleQuitClick = () => {
		setShowQuitDialog(true);
	};

	const { setView } = useRoomView();

	const handleConfirmQuit = () => {
		sendGameMessage("quit", null);
		setShowQuitDialog(false);
		// Switch to lobby view so after game-over modal, user is in lobby
		setView("lobby");
	};

	return (
		<>
			<div className="flex items-center justify-between gap-2 pt-4">
				{slot === "lobby" ? (
					<Button
						asChild
						variant={"link"}
						className="px-0 py-2.5 has-[>svg]:px-0"
						onClick={handleBackClick}
					>
						<Link href={"/lobby"}>
							<ChevronLeft />
							<span>Back</span>
						</Link>
					</Button>
				) : (
					<Button
						variant={"link"}
						className="px-0 py-2.5 text-red-500 hover:text-red-400 has-[>svg]:px-0"
						onClick={handleQuitClick}
					>
						<ChevronLeft />
						<span>Quit</span>
					</Button>
				)}
				<div className="flex items-center gap-1.5 sm:gap-2">
					{isConnecting ? (
						<div className="text-muted-foreground flex items-center gap-1.5 text-sm">
							<Wifi className="size-4 animate-pulse" />
							<span className="hidden sm:inline">
								Connecting...
							</span>
						</div>
					) : isConnected ? (
						<div
							className={cn(
								"flex items-center gap-1.5 text-sm font-medium",
								latency !== null && getLatencyColor(latency)
							)}
							title={
								latency !== null
									? getLatencyLabel(latency)
									: "Connected"
							}
						>
							<Wifi className="size-4" />
							{latency !== null && <span>{latency} ms</span>}
						</div>
					) : (
						<div className="flex items-center gap-1.5 text-sm text-red-500">
							<WifiOff className="size-4" />
							<span className="hidden sm:inline">
								Disconnected
							</span>
						</div>
					)}

					<ShareButton />
				</div>
			</div>

			{/* Quit Confirmation Dialog */}
			<Dialog open={showQuitDialog} onOpenChange={setShowQuitDialog}>
				<DialogContent className="sm:max-w-sm">
					<DialogHeader>
						<DialogTitle>Quit Game?</DialogTitle>
						<DialogDescription>
							You will be eliminated from the game and forfeit any
							potential winnings. This action cannot be undone.
						</DialogDescription>
					</DialogHeader>
					<DialogFooter>
						<Button
							variant="outline"
							onClick={() => setShowQuitDialog(false)}
						>
							Cancel
						</Button>
						<Button
							variant="destructive"
							onClick={handleConfirmQuit}
						>
							Quit Game
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</>
	);
}
