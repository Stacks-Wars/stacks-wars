import LobbyCard from "@/components/main/lobby-card";
import type { LobbyExtended } from "@/lib/definitions";

const lobby: LobbyExtended = {
	id: "1",
	name: "Test lobby Test lobby Test lobby Test lobby Test l",
	path: "test-lobby",
	gamePath: "test-game",
	description:
		" Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby Test lobby T",
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
	participantCount: 1,
	creatorWalletAddress: "SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D",
	gameImageUrl: "/images/lexi-wars.svg",
	gameMinPlayers: 0,
	gameMaxPlayers: 0,
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
