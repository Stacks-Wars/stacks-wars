"use client";
import { Button } from "@/components/ui/button";
import { useRouter } from "next/navigation";
import { useConnectUser } from "@/contexts/ConnectWalletContext";
import { Loader } from "lucide-react";
import { Lobby } from "@/types/schema/lobby";

interface OpenLobbyProps {
	lobby: Lobby;
}

export default function OpenLobby({ lobby }: OpenLobbyProps) {
	const router = useRouter();
	const { isConnecting, isConnected, handleConnect } = useConnectUser();
	return (
		<>
			{isConnected ? (
				<Button
					variant={"default"}
					className="w-full gap-1.5 "
					onClick={() => router.push(`/lobby/${lobby.id}`)}
				>
					{lobby.state === "waiting"
						? "Open Lobby"
						: lobby.state === "starting"
							? "Starting"
							: lobby.state === "inProgress"
								? "In progress (Spectate)"
								: lobby.state === "finished"
									? "Finished (View Results)"
									: "Loading"}
				</Button>
			) : (
				<Button
					onClick={handleConnect}
					type="button"
					disabled={isConnecting}
					className="w-full gap-1.5 "
				>
					{isConnecting && (
						<Loader
							className="h-4 w-4 mr-1 animate-spin"
							size={17}
						/>
					)}
					{isConnecting
						? "Connecting..."
						: "Connect wallet to open lobby"}
				</Button>
			)}
		</>
	);
}
