import NotFound from "@/app/not-found";
import GameCard from "@/components/main/game-card";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions";
import Image from "next/image";
import CreateLobbyForm from "./_components/create-lobby-form";

export default async function CreateLobbyPage({
	params,
}: {
	params: Promise<{ gamePath: string }>;
}) {
	const gamePath = (await params).gamePath;

	const game = await ApiClient.get<Game>(`/api/game/by-path/${gamePath}`);

	if (!game.data) {
		return <NotFound />;
	}

	return (
		<div className="container mx-auto px-4">
			<GameCard game={game.data} action="gamePage" />
			<div className="mx-auto max-w-4xl">
				<div className="flex items-center gap-4 mb-4 sm:mb-8">
					<div className="bg-primary/50 p-4.5 rounded-full inline-block">
						<Image
							src={"/icons/screen-users.svg"}
							alt="create lobby icon"
							width={16}
							height={16}
							className="text-foreground size-4"
						/>
					</div>
					<div>
						<h1 className="text-base sm:text-2xl font-medium">
							Create a Lobby
						</h1>
						<p className="text-xs sm:text-xl">
							Set up a new lobby and invite friends to join
						</p>
					</div>
				</div>
				<CreateLobbyForm {...game.data} />
			</div>
		</div>
	);
}
