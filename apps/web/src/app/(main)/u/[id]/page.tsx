"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { useUserStore } from "@/lib/stores/user";
import { ApiClient } from "@/lib/api/client";
import type { User, Game, CreateGameRequest } from "@/lib/definitions";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Loader2, Plus } from "lucide-react";
import Link from "next/link";

export default function UserProfilePage() {
	const params = useParams();
	const { user: currentUser } = useUserStore();
	const [user, setUser] = useState<User | null>(null);
	const [games, setGames] = useState<Game[]>([]);
	const [isLoading, setIsLoading] = useState(true);
	const [isCreateOpen, setIsCreateOpen] = useState(false);
	const [isCreating, setIsCreating] = useState(false);

	const isOwnProfile = currentUser?.id === params.id;

	const logout = async () => {
		try {
			await ApiClient.post("/api/logout");
			window.location.reload();
		} catch (error) {
			console.error("Logout failed:", error);
		}
	};

	const loadProfile = async () => {
		setIsLoading(true);
		try {
			const response = await ApiClient.get<User>(
				`/api/user/${params.id}`
			);
			if (response.data) {
				setUser(response.data);
			}

			// Load user's created games
			const gamesResponse = await ApiClient.get<Game[]>(
				`/api/game/by-creator/${params.id}`
			);
			if (gamesResponse.data) {
				setGames(gamesResponse.data);
			}
		} catch (error) {
			console.error("Failed to load profile:", error);
		} finally {
			setIsLoading(false);
		}
	};

	useEffect(() => {
		loadProfile();
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [params.id]);

	const handleCreateGame = async (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();
		setIsCreating(true);

		const formData = new FormData(e.currentTarget);
		const gameData: CreateGameRequest = {
			name: formData.get("name") as string,
			path: (formData.get("name") as string)
				.toLowerCase()
				.replace(/\s+/g, "-"),
			description: formData.get("description") as string,
			imageUrl: formData.get("imageUrl") as string,
			minPlayers: parseInt(formData.get("minPlayers") as string),
			maxPlayers: parseInt(formData.get("maxPlayers") as string),
			category: formData.get("category") as string,
		};

		try {
			const response = await ApiClient.post<Game>("/api/game", gameData);
			if (response.data) {
				setGames([...games, response.data]);
				setIsCreateOpen(false);
				e.currentTarget.reset();
			} else if (response.error) {
				console.error("Failed to create game:", response.error);
			}
		} catch (error) {
			console.error("Failed to create game:", error);
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

	if (!user) {
		return (
			<div className="container mx-auto px-4 py-8">
				<p>User not found</p>
			</div>
		);
	}

	return (
		<div className="container mx-auto px-4 py-8">
			<div className="flex flex-col gap-8">
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-4xl font-bold">
							{user.displayName || user.username || "Anonymous"}
						</h1>
						<p className="text-muted-foreground">
							{user.walletAddress}
						</p>
					</div>
					{isOwnProfile && (
						<div className="flex gap-2">
							<Button variant="outline" onClick={logout}>
								Logout
							</Button>
							<Link href="/games">
								<Button>Browse Games</Button>
							</Link>
						</div>
					)}
				</div>

				{isOwnProfile && (
					<Card>
						<CardHeader>
							<div className="flex items-center justify-between">
								<div>
									<CardTitle>Your Games</CardTitle>
									<CardDescription>
										Games you&apos;ve created
									</CardDescription>
								</div>
								<Dialog
									open={isCreateOpen}
									onOpenChange={setIsCreateOpen}
								>
									<DialogTrigger asChild>
										<Button>
											<Plus className="mr-2 h-4 w-4" />
											Create Game
										</Button>
									</DialogTrigger>
									<DialogContent>
										<DialogHeader>
											<DialogTitle>
												Create New Game
											</DialogTitle>
											<DialogDescription>
												Add a new game to the platform
											</DialogDescription>
										</DialogHeader>
										<form
											onSubmit={handleCreateGame}
											className="space-y-4"
										>
											<div>
												<Label htmlFor="name">
													Game Name
												</Label>
												<Input
													id="name"
													name="name"
													required
													placeholder="Coin Flip"
												/>
											</div>
											<div>
												<Label htmlFor="description">
													Description
												</Label>
												<Textarea
													id="description"
													name="description"
													required
													placeholder="A fast-paced guessing game..."
												/>
											</div>
											<div>
												<Label htmlFor="imageUrl">
													Image URL
												</Label>
												<Input
													id="imageUrl"
													name="imageUrl"
													required
													placeholder="https://example.com/image.png"
												/>
											</div>
											<div className="grid grid-cols-2 gap-4">
												<div>
													<Label htmlFor="minPlayers">
														Min Players
													</Label>
													<Input
														id="minPlayers"
														name="minPlayers"
														type="number"
														required
														min="1"
														defaultValue="2"
													/>
												</div>
												<div>
													<Label htmlFor="maxPlayers">
														Max Players
													</Label>
													<Input
														id="maxPlayers"
														name="maxPlayers"
														type="number"
														required
														min="2"
														defaultValue="10"
													/>
												</div>
											</div>
											<div>
												<Label htmlFor="category">
													Category
												</Label>
												<Input
													id="category"
													name="category"
													placeholder="Guessing Games"
												/>
											</div>
											<Button
												type="submit"
												disabled={isCreating}
												className="w-full"
											>
												{isCreating ? (
													<>
														<Loader2 className="mr-2 h-4 w-4 animate-spin" />
														Creating...
													</>
												) : (
													"Create Game"
												)}
											</Button>
										</form>
									</DialogContent>
								</Dialog>
							</div>
						</CardHeader>
						<CardContent>
							{games.length === 0 ? (
								<p className="text-center text-muted-foreground">
									No games created yet
								</p>
							) : (
								<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
									{games.map((game) => (
										<Link
											key={game.id}
											href={`/games/${game.path}`}
										>
											<Card className="cursor-pointer transition-colors hover:bg-accent">
												<CardHeader>
													<CardTitle>
														{game.name}
													</CardTitle>
													<CardDescription>
														{game.description?.substring(
															0,
															100
														)}
														...
													</CardDescription>
												</CardHeader>
											</Card>
										</Link>
									))}
								</div>
							)}
						</CardContent>
					</Card>
				)}
			</div>
		</div>
	);
}
