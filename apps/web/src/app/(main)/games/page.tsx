"use client";

import { useEffect, useState } from "react";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";
import Link from "next/link";
import Image from "next/image";

export default function GamesPage() {
	const [games, setGames] = useState<Game[]>([]);
	const [isLoading, setIsLoading] = useState(true);

	useEffect(() => {
		loadGames();
	}, []);

	const loadGames = async () => {
		setIsLoading(true);
		try {
			const response = await ApiClient.get<Game[]>("/api/game");
			if (response.data) {
				// Filter only active games
				setGames(response.data.filter((g) => g.isActive));
			}
		} catch (error) {
			console.error("Failed to load games:", error);
		} finally {
			setIsLoading(false);
		}
	};

	if (isLoading) {
		return (
			<div className="flex min-h-screen items-center justify-center">
				<Loader2 className="h-8 w-8 animate-spin" />
			</div>
		);
	}

	return (
		<div className="container mx-auto px-4 py-8">
			<div className="flex flex-col gap-8">
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-4xl font-bold">Browse Games</h1>
						<p className="text-muted-foreground">
							Choose a game to create a lobby
						</p>
					</div>
					<Link href="/">
						<Button variant="outline">Back to Home</Button>
					</Link>
				</div>

				{games.length === 0 ? (
					<Card>
						<CardContent className="pt-6">
							<p className="text-center text-muted-foreground">
								No games available yet. Create your first game!
							</p>
						</CardContent>
					</Card>
				) : (
					<div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
						{games.map((game) => (
							<Link key={game.id} href={`/games/${game.path}`}>
								<Card className="cursor-pointer transition-all hover:scale-105 hover:shadow-lg">
									<CardHeader>
										{game.imageUrl && (
											<div className="mb-4 aspect-video w-full overflow-hidden rounded-md">
												<Image
													src={game.imageUrl}
													alt={game.name}
													className="h-full w-full object-cover"
													width={60}
													height={60}
												/>
											</div>
										)}
										<CardTitle>{game.name}</CardTitle>
										<CardDescription>
											{game.description}
										</CardDescription>
									</CardHeader>
									<CardContent>
										<div className="flex items-center justify-between text-sm text-muted-foreground">
											<span>
												{game.minPlayers}-
												{game.maxPlayers} players
											</span>
											{game.category && (
												<span className="rounded-full bg-primary/10 px-2 py-1 text-xs">
													{game.category}
												</span>
											)}
										</div>
									</CardContent>
								</Card>
							</Link>
						))}
					</div>
				)}
			</div>
		</div>
	);
}
