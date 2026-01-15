"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { ChatMessage, PlayerState } from "@/lib/definitions";
import { formatAddress } from "@/lib/utils";
import { Send } from "lucide-react";
import { useState, useMemo } from "react";

interface ChatProps {
	messages: ChatMessage[];
	players: PlayerState[];
	onSendMessage: (content: string) => void;
	onAddReaction?: (messageId: string, emoji: string) => void;
	onRemoveReaction?: (messageId: string, emoji: string) => void;
	currentUserId?: string;
}

export default function Chat({
	messages,
	players,
	onSendMessage,
	onAddReaction,
	onRemoveReaction,
	currentUserId,
}: ChatProps) {
	const [newMessage, setNewMessage] = useState("");

	// Create a lookup map for player info
	const playerMap = useMemo(() => {
		const map = new Map<string, PlayerState>();
		players.forEach((p) => map.set(p.userId, p));
		return map;
	}, [players]);

	// Group reactions by emoji for display
	const groupReactions = (reactions: ChatMessage["reactions"]) => {
		const grouped: Record<string, string[]> = {};
		reactions.forEach((reaction) => {
			if (!grouped[reaction.emoji]) {
				grouped[reaction.emoji] = [];
			}
			grouped[reaction.emoji].push(reaction.userId);
		});
		return grouped;
	};

	const handleSend = () => {
		if (newMessage.trim()) {
			onSendMessage(newMessage);
			setNewMessage("");
		}
	};

	const handleKeyPress = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	};

	return (
		<div className="flex flex-col h-[60vh] max-h-150">
			<ScrollArea className="flex-1 pr-4">
				<div className="space-y-4">
					{messages.length === 0 ? (
						<p className="text-center text-muted-foreground py-20">
							No messages yet. Be the first to send a message!
						</p>
					) : (
						messages.map((msg) => {
							const sender = playerMap.get(msg.userId);
							const groupedReactions = groupReactions(
								msg.reactions
							);

							return (
								<div key={msg.messageId} className="space-y-1">
									<div className="flex items-baseline gap-2">
										<span className="font-medium text-sm">
											{sender?.displayName ||
												sender?.username ||
												(sender
													? formatAddress(
															sender.walletAddress
														)
													: "Unknown User")}
										</span>
										<span className="text-xs text-muted-foreground">
											{new Date(
												msg.createdAt
											).toLocaleTimeString([], {
												hour: "2-digit",
												minute: "2-digit",
											})}
										</span>
									</div>
									<p className="text-sm">{msg.content}</p>
									{Object.keys(groupedReactions).length >
										0 && (
										<div className="flex gap-1 flex-wrap">
											{Object.entries(
												groupedReactions
											).map(([emoji, userIds]) => (
												<button
													key={emoji}
													className="text-xs px-2 py-1 rounded-full bg-muted hover:bg-muted/80 transition-colors"
													onClick={() => {
														if (
															currentUserId &&
															userIds.includes(
																currentUserId
															)
														) {
															onRemoveReaction?.(
																msg.messageId,
																emoji
															);
														} else {
															onAddReaction?.(
																msg.messageId,
																emoji
															);
														}
													}}
												>
													{emoji} {userIds.length}
												</button>
											))}
										</div>
									)}
								</div>
							);
						})
					)}
				</div>
			</ScrollArea>

			<div className="flex gap-2 pt-4 border-t">
				<Input
					placeholder="Type a message..."
					value={newMessage}
					onChange={(e) => setNewMessage(e.target.value)}
					onKeyDown={handleKeyPress}
					className="flex-1"
				/>
				<Button
					onClick={handleSend}
					size="icon"
					disabled={!newMessage.trim()}
				>
					<Send className="size-4" />
				</Button>
			</div>
		</div>
	);
}
