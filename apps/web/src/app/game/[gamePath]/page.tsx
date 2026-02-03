import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions/game";
import type { LobbyInfo, PaginatedLobbiesResponse } from "@/lib/definitions";
import GameLobbies from "../_components/game-lobbies";
import Image from "next/image";

interface PageProps {
	params: Promise<{ gamePath: string }>;
}

async function getGame(identifier: string): Promise<Game> {
	try {
		const res = await ApiClient.get<Game>(`/api/game/${identifier}`);
		if (!res.data) {
			throw new Error("No game data received");
		}
		return res.data;
	} catch (error) {
		console.error("Failed to fetch game:", error);
		notFound();
	}
}

async function getGameLobbies(identifier: string): Promise<{
	lobbies: LobbyInfo[];
	total: number;
}> {
	try {
		const res = await ApiClient.get<PaginatedLobbiesResponse>(
			`/api/game/${identifier}/lobbies?statuses=waiting,starting,inProgress&limit=6&offset=0`
		);
		if (!res.data) {
			return { lobbies: [], total: 0 };
		}
		return {
			lobbies: res.data.data,
			total: res.data.total,
		};
	} catch (error) {
		console.error("Failed to fetch game lobbies:", error);
		return { lobbies: [], total: 0 };
	}
}

export async function generateMetadata({
	params,
}: PageProps): Promise<Metadata> {
	const gamePath = (await params).gamePath;

	try {
		const game = await getGame(gamePath);
		return {
			title: `${game.name}`,
			description: game.description || `Play ${game.name} on Stacks Wars`,
			openGraph: {
				images: [
					{
						url: game.imageUrl,
						width: 1200,
						height: 630,
						alt: game.name,
					},
				],
			},
			twitter: {
				images: [game.imageUrl],
			},
		};
	} catch {
		return {
			title: "Game",
			description: "Play exciting games on Stacks Wars",
		};
	}
}

export default async function GamePage({ params }: PageProps) {
	const gamePath = (await params).gamePath;

	const [game, { lobbies, total }] = await Promise.all([
		getGame(gamePath),
		getGameLobbies(gamePath),
	]);

	return (
		<div className="container mx-auto px-4 py-8">
			{/* Game Header */}
			<div className="mb-8 text-center sm:mb-12">
				<div className="mx-auto mb-6 w-full max-w-md sm:max-w-lg">
					<Image
						src={game.imageUrl}
						alt={game.name}
						width={400}
						height={200}
						className="h-auto w-full rounded-2xl"
						priority
					/>
				</div>
				<h1 className="mb-4 text-3xl font-bold sm:text-4xl lg:text-5xl">
					{game.name}
				</h1>
				<p className="text-muted-foreground mx-auto max-w-2xl text-lg sm:text-xl">
					{game.description}
				</p>
			</div>

			{/* Game Details */}
			<div className="bg-background/50 mb-12 rounded-2xl p-4 sm:p-6">
				<div className="mx-auto max-w-4xl">
					<h2 className="mb-4 text-xl font-semibold sm:text-2xl">
						Game Details
					</h2>
					<div className="grid grid-cols-1 gap-2 text-sm sm:grid-cols-2 sm:text-base">
						<p>
							<span className="font-medium">Players:</span>{" "}
							{game.minPlayers} - {game.maxPlayers}
						</p>
						<p>
							<span className="font-medium">Category:</span>{" "}
							{game.category && game.category.length > 0
								? game.category.join(", ")
								: "Game"}
						</p>
					</div>
				</div>
			</div>

			{/* Active Lobbies */}
			<GameLobbies
				gameIdentifier={gamePath}
				initialLobbies={lobbies}
				initialTotal={total}
			/>
		</div>
	);
}
