import NotFound from "@/app/not-found";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions";

export default async function DefaultGamePage({
	params,
}: {
	params: Promise<{ gamePath: string }>;
}) {
	const gamePath = (await params).gamePath;

	const game = await ApiClient.get<Game>(`/api/game/by-path/${gamePath}`);

	if (!game.data) {
		return <NotFound />;
	}

	return <div>DefaultGamePage</div>;
}
