import RoomContent from "../room-content";

export default async function RoomLayout({
	children,
	lobby,
	game,
	params,
}: {
	children: React.ReactNode;
	lobby: React.ReactNode;
	game: React.ReactNode;
	params: Promise<{ lobbyPath: string }>;
}) {
	const lobbyPath = (await params).lobbyPath;

	return (
		<RoomContent lobby={lobby} game={game} lobbyPath={lobbyPath}>
			{children}
		</RoomContent>
	);
}
