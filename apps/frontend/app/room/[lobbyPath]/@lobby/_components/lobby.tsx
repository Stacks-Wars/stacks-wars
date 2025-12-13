/**
 * Lobby Component
 *
 * Displays lobby information, players, chat, and join requests.
 * Works with any game through the unified game engine.
 */

"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Users, Clock, Send, Crown, UserPlus, X, Check } from "lucide-react";
import {
	LobbyExtended,
	PlayerState,
	JoinRequest,
	ChatMessage,
} from "@/lib/definitions";

interface LobbyProps {
	lobby: LobbyExtended;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
	onSendMessage: (type: string, payload?: unknown) => void;
}

export function Lobby({
	lobby,
	players,
	joinRequests,
	chatHistory,
	onSendMessage,
}: LobbyProps) {
	const [message, setMessage] = useState("");

	const handleSendMessage = () => {
		if (message.trim()) {
			onSendMessage("sendMessage", { content: message });
			setMessage("");
		}
	};

	const handleApproveJoin = (playerId: string) => {
		onSendMessage("approveJoin", { playerId });
	};

	const handleRejectJoin = (playerId: string) => {
		onSendMessage("rejectJoin", { playerId });
	};

	const handleKickPlayer = (playerId: string) => {
		onSendMessage("kick", { playerId });
	};

	return (
		<div className="container mx-auto px-4 py-8">
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<Users className="h-5 w-5" />
						{lobby.name}
					</CardTitle>
				</CardHeader>
				<CardContent className="space-y-6">
					<div className="flex items-center justify-between">
						<div className="flex items-center gap-2 text-sm text-muted-foreground">
							<Clock className="h-4 w-4" />
							Status:{" "}
							<Badge variant="outline">{lobby.status}</Badge>
						</div>
						<div className="text-sm text-muted-foreground">
							{players.length} / {lobby.participantCount} players
						</div>
					</div>

					{/* Players */}
					<div>
						<h3 className="mb-3 font-semibold">Players</h3>
						<div className="space-y-2">
							{players.map((player) => (
								<div
									key={player.userId}
									className="flex items-center justify-between rounded-lg border p-3"
								>
									<div className="flex items-center gap-2">
										{player.isCreator && (
											<Crown className="h-4 w-4 text-yellow-500" />
										)}
										<span className="font-mono text-sm">
											{player.userId}
										</span>
									</div>
									{lobby.creatorId && (
										<Button
											variant="ghost"
											size="sm"
											onClick={() =>
												handleKickPlayer(player.userId)
											}
											disabled={player.isCreator}
										>
											<X className="h-4 w-4" />
										</Button>
									)}
								</div>
							))}
						</div>
					</div>

					{/* Join Requests */}
					{joinRequests.length > 0 && (
						<div>
							<h3 className="mb-3 flex items-center gap-2 font-semibold">
								<UserPlus className="h-4 w-4" />
								Join Requests ({joinRequests.length})
							</h3>
							<div className="space-y-2">
								{joinRequests.map((request) => (
									<div
										key={request.playerId}
										className="flex items-center justify-between rounded-lg border p-3"
									>
										<span className="font-mono text-sm">
											{request.playerId}
										</span>
										<div className="flex gap-2">
											<Button
												variant="outline"
												size="sm"
												onClick={() =>
													handleApproveJoin(
														request.playerId
													)
												}
											>
												<Check className="h-4 w-4" />
											</Button>
											<Button
												variant="outline"
												size="sm"
												onClick={() =>
													handleRejectJoin(
														request.playerId
													)
												}
											>
												<X className="h-4 w-4" />
											</Button>
										</div>
									</div>
								))}
							</div>
						</div>
					)}

					{/* Chat */}
					<div>
						<h3 className="mb-3 font-semibold">Chat</h3>
						<div className="rounded-md border">
							<div className="h-64 space-y-2 overflow-y-auto p-4">
								{chatHistory.length === 0 ? (
									<p className="text-sm text-muted-foreground">
										No messages yet
									</p>
								) : (
									chatHistory.map((msg) => (
										<div
											key={msg.id}
											className="rounded-lg bg-muted p-2"
										>
											<div className="mb-1 flex items-center gap-2">
												<span className="font-mono text-xs font-medium">
													{msg.senderId.slice(0, 8)}
													...
												</span>
												<span className="text-xs text-muted-foreground">
													{new Date(
														msg.timestamp
													).toLocaleTimeString()}
												</span>
											</div>
											<p className="text-sm">
												{msg.content}
											</p>
										</div>
									))
								)}
							</div>
							<div className="flex gap-2 border-t p-3">
								<Input
									value={message}
									onChange={(e) => setMessage(e.target.value)}
									onKeyDown={(e) => {
										if (e.key === "Enter") {
											handleSendMessage();
										}
									}}
									placeholder="Type a message..."
								/>
								<Button
									onClick={handleSendMessage}
									disabled={!message.trim()}
								>
									<Send className="h-4 w-4" />
								</Button>
							</div>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
