"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { apiRequest } from "@/lib/api";
import { toast } from "sonner";
import {
	Users,
	Clock,
	Play,
	Loader2,
	ExternalLink,
	Gamepad2,
} from "lucide-react";
import { PlayerLobbyInfo } from "@/types/schema/lobby";
import Link from "next/link";
import { formatNumber } from "@/lib/utils";

interface ActiveLobbiesProps {
	identifier: string;
}

export default function ActiveLobbies({ identifier }: ActiveLobbiesProps) {
	const [activeLobbies, setActiveLobbies] = useState<PlayerLobbyInfo[]>([]);
	const [loading, setLoading] = useState(true);

	useEffect(() => {
		const fetchActiveLobbies = async () => {
			try {
				const response = await apiRequest<PlayerLobbyInfo[]>({
					path: `/user/lobbies?identifier=${encodeURIComponent(identifier)}&lobby_state=waiting,starting,inProgress`,
					method: "GET",
					auth: false,
				});
				setActiveLobbies(response);
			} catch (error) {
				console.error("Failed to fetch active lobbies:", error);
				toast.error("Failed to load active lobbies");
			} finally {
				setLoading(false);
			}
		};

		fetchActiveLobbies();
	}, [identifier]);

	const getStateIcon = (state: string) => {
		switch (state) {
			case "waiting":
				return <Clock className="h-4 w-4" />;
			case "starting":
				return <Loader2 className="h-4 w-4 animate-spin" />;
			case "inProgress":
				return <Play className="h-4 w-4" />;
			default:
				return <Gamepad2 className="h-4 w-4" />;
		}
	};

	const getStateColor = (state: string) => {
		switch (state) {
			case "waiting":
				return "bg-yellow-500/10 text-yellow-700 dark:text-yellow-400";
			case "starting":
				return "bg-blue-500/10 text-blue-700 dark:text-blue-400";
			case "inProgress":
				return "bg-green-500/10 text-green-700 dark:text-green-400";
			default:
				return "bg-gray-500/10 text-gray-700 dark:text-gray-400";
		}
	};

	const formatDate = (dateString: string) => {
		return new Date(dateString).toLocaleDateString("en-US", {
			month: "short",
			day: "numeric",
			hour: "2-digit",
			minute: "2-digit",
		});
	};

	const getGameDisplayName = (gameType: string) => {
		switch (gameType) {
			case "lexi-wars":
				return "Lexi Wars";
			default:
				return gameType.charAt(0).toUpperCase() + gameType.slice(1);
		}
	};

	if (loading) {
		return (
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<Gamepad2 className="h-5 w-5 text-primary" />
						Active Lobbies
					</CardTitle>
				</CardHeader>
				<CardContent className="flex items-center justify-center py-8">
					<Loader2 className="h-6 w-6 animate-spin" />
				</CardContent>
			</Card>
		);
	}

	if (activeLobbies.length === 0) {
		return (
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<Gamepad2 className="h-5 w-5 text-primary" />
						Active Lobbies
					</CardTitle>
				</CardHeader>
				<CardContent className="text-center py-8">
					<div className="text-muted-foreground">
						<Users className="h-12 w-12 mx-auto mb-2 opacity-50" />
						<p>No active lobbies</p>
						<p className="text-sm">
							This player is not currently in any games
						</p>
					</div>
				</CardContent>
			</Card>
		);
	}

	return (
		<Card>
			<CardHeader>
				<CardTitle className="flex items-center gap-2">
					<Gamepad2 className="h-5 w-5 text-primary" />
					Active Lobbies
					<Badge variant="secondary" className="ml-auto">
						{activeLobbies.length}
					</Badge>
				</CardTitle>
			</CardHeader>
			<CardContent className="space-y-4">
				{activeLobbies.map((lobby) => (
					<div
						key={lobby.id}
						className="flex items-center justify-between p-4 border rounded-lg bg-muted/50 hover:bg-muted/80 transition-colors"
					>
						<div className="flex-1 min-w-0">
							<div className="flex items-center gap-2 mb-2">
								<h4 className="font-medium truncate">
									{lobby.name}
								</h4>
								<Badge
									variant="outline"
									className={`shrink-0 ${getStateColor(lobby.state)}`}
								>
									<div className="flex items-center gap-1">
										{getStateIcon(lobby.state)}
										{lobby.state === "waiting"
											? "Waiting"
											: "In Progress"}
									</div>
								</Badge>
							</div>

							<div className="flex items-center gap-4 text-sm text-muted-foreground">
								<div className="flex items-center gap-1">
									<Gamepad2 className="h-3 w-3" />
									{getGameDisplayName(lobby.game.name)}
								</div>
								<div className="flex items-center gap-1">
									<Users className="h-3 w-3" />
									{lobby.participants} players
								</div>
								<div className="flex items-center gap-1">
									<Clock className="h-3 w-3" />
									{formatDate(lobby.createdAt)}
								</div>
								{lobby.entryAmount && (
									<div className="text-green-600 font-medium">
										{formatNumber(lobby.entryAmount)}{" "}
										{lobby.tokenSymbol}
									</div>
								)}
							</div>

							{lobby.description && (
								<p className="text-sm text-muted-foreground mt-1 truncate">
									{lobby.description}
								</p>
							)}
						</div>

						<Link href={`/lobby/${lobby.id}`}>
							<Button variant="outline" className="shrink-0">
								<ExternalLink className="h-4 w-4 mr-2" />
								View Lobby
							</Button>
						</Link>
					</div>
				))}
			</CardContent>
		</Card>
	);
}
