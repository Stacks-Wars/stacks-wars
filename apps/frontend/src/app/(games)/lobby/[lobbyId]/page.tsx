import { apiRequest } from "@/lib/api";
import { getClaimFromJwt } from "@/lib/getClaimFromJwt";
import Lobby from "./_components/lobby";
import RequireAuth from "@/components/require-auth";
import NotFound from "@/app/not-found";
import { LobbyExtended } from "@/types/schema/lobby";

export default async function LobbyDetailPage({
	params,
}: {
	params: Promise<{ lobbyId: string }>;
}) {
	const lobbyId = (await params).lobbyId;

	const userId = await getClaimFromJwt<string>("sub");

	if (!userId) {
		return <RequireAuth />;
	}

	const lobbyExtended = await apiRequest<LobbyExtended>({
		path: `/lobby/extended/${lobbyId}?player_state=joined`,
		tag: "lobbyExtended",
	});

	if (!lobbyExtended) {
		return <NotFound />;
	}

	return (
		<Lobby
			lobby={lobbyExtended.lobby}
			players={lobbyExtended.players}
			userId={userId}
			lobbyId={lobbyId}
			game={lobbyExtended.lobby.game}
		/>
	);
}
