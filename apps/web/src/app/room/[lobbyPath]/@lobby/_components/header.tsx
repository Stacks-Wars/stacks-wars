import Link from "next/link";
import { Button } from "@/components/ui/button";
import { ChevronLeft, Gamepad2, Wifi, WifiOff } from "lucide-react";
import { useRoomView } from "@/lib/contexts/room-view-context";
import { useRoom } from "@/lib/contexts/room-context";
import ShareButton from "./share-button";
import type { LobbyExtended } from "@/lib/definitions";
import { cn } from "@/lib/utils";

interface RoomHeaderProps {
	lobby: LobbyExtended;
	isConnecting: boolean;
	isConnected: boolean;
	latency: number | null;
}

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

export default function RoomHeader({
	lobby,
	isConnected,
	isConnecting,
	latency,
}: RoomHeaderProps) {
	const { setView } = useRoomView();
	const { disconnect } = useRoom();

	const handleBackClick = () => {
		// Disconnect WebSocket before navigating away
		disconnect();
	};

	return (
		<div className="flex items-center justify-between gap-2">
			<Button
				asChild
				variant={"link"}
				className="has-[>svg]:px-0 px-0 py-2.5"
				onClick={handleBackClick}
			>
				<Link href={"/lobby"}>
					<ChevronLeft />
					<span>Back</span>
				</Link>
			</Button>
			<div className="flex items-center gap-1.5 sm:gap-2">
				{isConnecting ? (
					<div className="flex items-center gap-1.5 text-sm text-muted-foreground">
						<Wifi className="size-4 animate-pulse" />
						<span className="hidden sm:inline">Connecting...</span>
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
						<span className="hidden sm:inline">Disconnected</span>
					</div>
				)}

				<ShareButton lobbyPath={lobby.path} />
			</div>
		</div>
	);
}
