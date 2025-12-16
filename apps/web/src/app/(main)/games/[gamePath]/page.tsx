"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { ApiClient } from "@/lib/api/client";
import type { Game, Lobby, CreateLobbyRequest } from "@/lib/definitions";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Loader2, ArrowLeft } from "lucide-react";
import Link from "next/link";
import Image from "next/image";

export default function GameDetailPage() {
	const params = useParams();
	const router = useRouter();
	const [game, setGame] = useState<Game | null>(null);
	const [isLoading, setIsLoading] = useState(true);
	const [isCreating, setIsCreating] = useState(false);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		loadGame();
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	const loadGame = async () => {
		setIsLoading(true);
		try {
			const response = await ApiClient.get<Game>(
				`/api/game/by-path/${params.gamePath}`
			);
			if (response.data) {
				setGame(response.data);
			} else {
				setError("Game not found");
			}
		} catch (error) {
			console.error("Failed to load game:", error);
			setError("Failed to load game");
		} finally {
			setIsLoading(false);
		}
	};

	const handleCreateLobby = async (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();
		if (!game) return;

		setIsCreating(true);
		setError(null);

		const formData = new FormData(e.currentTarget);
		const lobbyData: CreateLobbyRequest = {
			name: formData.get("name") as string,
			description: formData.get("description") as string,
			gameId: game.id,
			gamePath: game.path,
			isPrivate: false,
			isSponsored: false,
		};

		try {
			const response = await ApiClient.post<Lobby>(
				"/api/lobby",
				lobbyData
			);
			if (response.error || !response.data) {
				throw new Error(response.error || "Failed to create lobby");
			}

			// Redirect to the lobby room
			router.push(`/room/${response.data.path}`);
		} catch (err) {
			setError(
				err instanceof Error ? err.message : "Failed to create lobby"
			);
		} finally {
			setIsCreating(false);
		}
	};

	if (isLoading) {
		return (
			<div className="flex min-h-screen items-center justify-center">
				<Loader2 className="h-8 w-8 animate-spin" />
			</div>
		);
	}

	if (!game) {
		return (
			<div className="container mx-auto px-4 py-8">
				<Card>
					<CardContent className="pt-6">
						<p className="text-center text-muted-foreground">
							{error || "Game not found"}
						</p>
						<div className="mt-4 flex justify-center">
							<Link href="/games">
								<Button variant="outline">Back to Games</Button>
							</Link>
						</div>
					</CardContent>
				</Card>
			</div>
		);
	}

	return (
		<div className="container mx-auto px-4 py-8">
			<div className="flex flex-col gap-8">
				<Link href="/games">
					<Button variant="ghost" size="sm">
						<ArrowLeft className="mr-2 h-4 w-4" />
						Back to Games
					</Button>
				</Link>

				<div className="grid gap-8 lg:grid-cols-2">
					{/* Game Info */}
					<Card>
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
							<div className="grid gap-2 text-sm">
								<div className="flex justify-between">
									<span className="text-muted-foreground">
										Players
									</span>
									<span className="font-medium">
										{game.minPlayers}-{game.maxPlayers}
									</span>
								</div>
								{game.category && (
									<div className="flex justify-between">
										<span className="text-muted-foreground">
											Category
										</span>
										<span className="font-medium">
											{game.category}
										</span>
									</div>
								)}
							</div>
						</CardContent>
					</Card>

					{/* Lobby Creation Form */}
					<Card>
						<CardHeader>
							<CardTitle>Create Lobby</CardTitle>
							<CardDescription>
								Start a new game lobby (Free, no entry fee)
							</CardDescription>
						</CardHeader>
						<CardContent>
							<form
								onSubmit={handleCreateLobby}
								className="space-y-4"
							>
								<div>
									<Label htmlFor="name">Lobby Name</Label>
									<Input
										id="name"
										name="name"
										required
										placeholder="My Awesome Lobby"
									/>
								</div>
								<div>
									<Label htmlFor="description">
										Description (Optional)
									</Label>
									<Textarea
										id="description"
										name="description"
										placeholder="Come join for a fun game!"
										rows={3}
									/>
								</div>

								{error && (
									<div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
										{error}
									</div>
								)}

								<Button
									type="submit"
									disabled={isCreating}
									className="w-full"
								>
									{isCreating ? (
										<>
											<Loader2 className="mr-2 h-4 w-4 animate-spin" />
											Creating Lobby...
										</>
									) : (
										"Create Lobby & Start"
									)}
								</Button>
							</form>
						</CardContent>
					</Card>
				</div>
			</div>
		</div>
	);
}
