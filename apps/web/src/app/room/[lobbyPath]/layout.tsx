import dynamic from "next/dynamic";
import type { Metadata } from "next";
import { ApiClient } from "@/lib/api/client";
import type { Lobby } from "@/lib/definitions";

const RoomContent = dynamic(() => import("./room-content"));

interface LayoutProps {
	children: React.ReactNode;
	lobby: React.ReactNode;
	game: React.ReactNode;
	params: Promise<{ lobbyPath: string }>;
}

async function getLobby(lobbyPath: string): Promise<Lobby> {
	try {
		const res = await ApiClient.get<Lobby>(`/api/lobbies/${lobbyPath}`);
		if (!res.data) {
			throw new Error("No lobby data received");
		}
		return res.data;
	} catch (error) {
		console.error("Failed to fetch lobby:", error);
		throw error;
	}
}

export async function generateMetadata({
	params,
}: LayoutProps): Promise<Metadata> {
	const lobbyPath = (await params).lobbyPath;

	try {
		const lobby = await getLobby(lobbyPath);
		return {
			title: `${lobby.name} lobby`,
			description:
				lobby.description ||
				`Join the ${lobby.name} lobby on Stacks Wars`,
		};
	} catch {
		return {
			title: "Room",
			description: "Join a game room on Stacks Wars",
		};
	}
}

export default async function RoomLayout({
	children,
	lobby,
	game,
	params,
}: LayoutProps) {
	const lobbyPath = (await params).lobbyPath;

	return (
		<main className="flex min-h-screen flex-col">
			<RoomContent lobby={lobby} game={game} lobbyPath={lobbyPath}>
				{children}
			</RoomContent>
		</main>
	);
}
