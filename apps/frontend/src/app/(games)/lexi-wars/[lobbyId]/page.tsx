import { getClaimFromJwt } from "@/lib/getClaimFromJwt";
import LexiWars from "./_components/lexi-wars";
import { apiRequest } from "@/lib/api";
import RequireAuth from "@/components/require-auth";
import { Lobby } from "@/types/schema/lobby";

export default async function LexiWarsPage({
	params,
}: {
	params: Promise<{ lobbyId: string }>;
}) {
	const lobbyId = (await params).lobbyId;

	const userId = await getClaimFromJwt<string>("sub");

	if (!userId) {
		return <RequireAuth />;
	}

	const lobby = await apiRequest<Lobby>({
		path: `lobby/${lobbyId}`,
		cache: "no-store",
	});

	return <LexiWars lobbyId={lobbyId} userId={userId} lobby={lobby} />;
}
