export default function RoomLayout({
	children,
	lobby,
	game,
}: {
	children: React.ReactNode;
	lobby: React.ReactNode;
	game: React.ReactNode;
}) {
	return (
		<div>
			{lobby}
			{game}
			{children}
		</div>
	);
}
