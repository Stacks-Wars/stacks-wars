import { Button } from "@/components/ui/button";
import { ChevronLeft } from "lucide-react";
import Link from "next/link";
import ShareButton from "./share-button";
import type { Game, JoinRequest, PlayerState } from "@/lib/definitions";
import GameCard from "@/components/main/game-card";
import Participants from "./participants";

const game: Game = {
	id: "id",
	name: "name",
	path: "path",
	description: "description",
	imageUrl: "/images/lexi-wars.svg",
	minPlayers: 0,
	maxPlayers: 0,
	creatorId: "",
	isActive: true,
	createdAt: "",
	updatedAt: "",
};

const player: PlayerState = {
	userId: "userId",
	lobbyId: "lobbyId",
	status: "joined",
	state: "accepted",
	walletAddress: "walletAddress",
	displayName:
		"Flames walletAddresswalletAddresswalle tAddresswalletAddresswalletAddress",
	username:
		"usernamewalletAddresswalletAddresswalletAddresswalletAddresswalletAddress",
	trustRating: 5,
	txId: "txId",
	rank: 1,
	prize: 100,
	claimState: "unclaimed",
	lastPing: Date.now(),
	joinedAt: 0,
	updatedAt: 0,
	isCreator: false,
};

const pendingPlayer: JoinRequest = {
	playerId: "playerId",
	walletAddress: "walletAddress",
	username: "username",
	displayName: "Flames",
	trustRating: 5,
	state: "pending",
};

const isCreator = true;

export default function Lobby() {
	return (
		<div className="space-y-4 sm:space-y-8">
			<div className="flex items-center justify-between">
				<Button
					asChild
					variant={"link"}
					className="has-[>svg]:px-0 px-0 py-2.5"
				>
					<Link href={"/lobby"}>
						<ChevronLeft />
						Back to Lobby
					</Link>
				</Button>
				<div className="flex items-center gap-4">
					<p className="text-base font-medium">400ms</p>
					<ShareButton lobbyPath={"example-lobby"} />
				</div>
			</div>
			<GameCard game={game} open="gamePage" />
			<Participants
				players={[player]}
				pendingPlayers={[pendingPlayer]}
				isCreator={isCreator}
			/>
		</div>
	);
}
