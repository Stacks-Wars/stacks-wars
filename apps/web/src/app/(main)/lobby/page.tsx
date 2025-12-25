import LobbyCard from "@/components/main/lobby-card";
import type { Lobby } from "@/lib/definitions";

const lobby: Lobby = {
	id: "1",
	name: "Test Lobby",
	path: "test-lobby",
	gamePath: "test-game",
	description: "This is a test lobby",
	gameId: "game1",
	creatorId: "user1",
	entryAmount: 10,
	currentAmount: 10,
	tokenSymbol: "STX",
	tokenContractId: "contract1",
	contractAddress: "address1",
	isPrivate: false,
	isSponsored: false,
	status: "waiting",
	createdAt: new Date().toISOString(),
	updatedAt: new Date().toISOString(),
};

export default function LobbyPage() {
	return (
		<div className="container mx-auto px-4">
			<h1 className="text-2xl lg:text-5xl font-bold text-center py-4 lg:py-15">
				Available Lobbies
			</h1>
			<LobbyCard lobby={lobby} />
		</div>
	);
}
