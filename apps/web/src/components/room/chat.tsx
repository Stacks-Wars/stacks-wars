"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import type { ChatMessage, PlayerState } from "@/lib/definitions";
import {
	useChatHistory,
	usePlayers,
	useIsActionLoading,
} from "@/lib/stores/room";
import { useUser } from "@/lib/stores/user";
import { useRoom } from "@/lib/contexts/room-context";
import { cn, displayUserIdentifier } from "@/lib/utils";
import { MessageCircle, Send, Smile } from "lucide-react";
import { useState, useMemo, useRef, useEffect } from "react";

const QUICK_REACTIONS = ["👍", "❤️", "😂", "🔥", "👏", "😮"];

interface ChatDialogProps {
	className?: string;
	buttonClassName?: string;
	buttonSize?: "default" | "sm" | "lg" | "icon";
	buttonVariant?: "default" | "outline" | "ghost" | "secondary";
}

export default function ChatDialog({
	className,
	buttonClassName,
	buttonSize = "icon",
	buttonVariant = "outline",
}: ChatDialogProps) {
	const [open, setOpen] = useState(false);
	const messages = useChatHistory();
	const players = usePlayers();
	const user = useUser();
	const { sendLobbyMessage } = useRoom();
	const scrollRef = useRef<HTMLDivElement>(null);
	const [showReactionPicker, setShowReactionPicker] = useState<string | null>(
		null
	);

	const [newMessage, setNewMessage] = useState("");
	const isSending = useIsActionLoading("sendMessage");

	// Auto-scroll to bottom when new messages arrive
	useEffect(() => {
		if (scrollRef.current) {
			scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
		}
	}, [messages]);

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
			sendLobbyMessage({ type: "sendMessage", content: newMessage });
			setNewMessage("");
		}
	};

	const handleKeyPress = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	};

	const handleAddReaction = (messageId: string, emoji: string) => {
		sendLobbyMessage({ type: "addReaction", messageId, emoji });
		setShowReactionPicker(null);
	};

	const handleRemoveReaction = (messageId: string, emoji: string) => {
		sendLobbyMessage({ type: "removeReaction", messageId, emoji });
	};

	const toggleReaction = (
		messageId: string,
		emoji: string,
		userIds: string[]
	) => {
		if (user?.id && userIds.includes(user.id)) {
			handleRemoveReaction(messageId, emoji);
		} else {
			handleAddReaction(messageId, emoji);
		}
	};

	const getDisplayName = (senderId: string) => {
		const sender = playerMap.get(senderId);
		return sender ? displayUserIdentifier(sender) : "Unknown";
	};

	const isOwnMessage = (senderId: string) => user?.id === senderId;

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<Tooltip>
				<TooltipTrigger asChild>
					<DialogTrigger asChild>
						<Button
							variant={buttonVariant}
							size={buttonSize}
							className={cn(
								"relative rounded-full",
								buttonClassName
							)}
						>
							<MessageCircle className="size-5" />
							{messages.length > 0 && (
								<span className="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-primary text-[10px] font-medium text-primary-foreground">
									{messages.length > 99
										? "99+"
										: messages.length}
								</span>
							)}
						</Button>
					</DialogTrigger>
				</TooltipTrigger>
				<TooltipContent>
					<p>Chat</p>
				</TooltipContent>
			</Tooltip>

			<DialogContent
				className={cn(
					"flex flex-col max-w-md h-[70vh] max-h-150 p-0 gap-0",
					className
				)}
			>
				<DialogHeader className="px-4 py-3 border-b shrink-0">
					<DialogTitle className="flex items-center gap-2">
						<MessageCircle className="size-5" />
						Lobby Chat
						<span className="text-xs font-normal text-muted-foreground">
							({players.length} online)
						</span>
					</DialogTitle>
				</DialogHeader>

				{/* Messages Area */}
				<ScrollArea ref={scrollRef} className="flex-1 px-4">
					<div className="py-4 space-y-3">
						{messages.length === 0 ? (
							<div className="flex flex-col items-center justify-center py-12 text-center">
								<MessageCircle className="size-12 text-muted-foreground/30 mb-3" />
								<p className="text-sm text-muted-foreground">
									No messages yet
								</p>
								<p className="text-xs text-muted-foreground/70">
									Be the first to say something!
								</p>
							</div>
						) : (
							messages.map((msg) => {
								const groupedReactions = groupReactions(
									msg.reactions
								);
								const isOwn = isOwnMessage(msg.userId);

								return (
									<div
										key={msg.messageId}
										className={cn(
											"group flex flex-col gap-1",
											isOwn && "items-end"
										)}
									>
										{/* Sender name & time */}
										<div
											className={cn(
												"flex items-center gap-2 px-1",
												isOwn && "flex-row-reverse"
											)}
										>
											<span className="text-xs font-medium text-muted-foreground">
												{isOwn
													? "You"
													: getDisplayName(
															msg.userId
														)}
											</span>
											<span className="text-[10px] text-muted-foreground/60">
												{new Date(
													msg.createdAt
												).toLocaleTimeString([], {
													hour: "2-digit",
													minute: "2-digit",
												})}
											</span>
										</div>

										{/* Message bubble */}
										<div
											className={cn(
												"relative max-w-[85%] w-fit rounded-2xl px-3 py-2 text-sm",
												isOwn
													? "bg-primary text-primary-foreground rounded-br-md"
													: "bg-muted rounded-bl-md"
											)}
										>
											{msg.content}

											{/* Reaction button (shows on hover) */}
											<button
												onClick={() =>
													setShowReactionPicker(
														showReactionPicker ===
															msg.messageId
															? null
															: msg.messageId
													)
												}
												className={cn(
													"absolute -bottom-2 opacity-0 group-hover:opacity-100 transition-opacity",
													"flex h-6 w-6 items-center justify-center rounded-full bg-background border shadow-sm",
													"hover:bg-muted",
													isOwn
														? "left-0 -translate-x-1/2"
														: "right-0 translate-x-1/2"
												)}
											>
												<Smile className="size-3 text-muted-foreground" />
											</button>

											{/* Quick reaction picker */}
											{showReactionPicker ===
												msg.messageId && (
												<div
													className={cn(
														"absolute -bottom-9 z-10 flex gap-1 rounded-full bg-background border shadow-lg px-2 py-1",
														isOwn
															? "right-0"
															: "left-0"
													)}
												>
													{QUICK_REACTIONS.map(
														(emoji) => (
															<button
																key={emoji}
																onClick={() =>
																	handleAddReaction(
																		msg.messageId,
																		emoji
																	)
																}
																className="hover:scale-125 transition-transform text-base"
															>
																{emoji}
															</button>
														)
													)}
												</div>
											)}
										</div>

										{/* Reactions */}
										{Object.keys(groupedReactions).length >
											0 && (
											<div
												className={cn(
													"flex gap-1 flex-wrap mt-1",
													isOwn && "justify-end"
												)}
											>
												{Object.entries(
													groupedReactions
												).map(([emoji, userIds]) => (
													<button
														key={emoji}
														className={cn(
															"text-xs px-2 py-0.5 rounded-full border transition-colors",
															user?.id &&
																userIds.includes(
																	user.id
																)
																? "bg-primary/10 border-primary/30"
																: "bg-muted/50 border-transparent hover:border-muted-foreground/20"
														)}
														onClick={() =>
															toggleReaction(
																msg.messageId,
																emoji,
																userIds
															)
														}
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

				{/* Input Area */}
				<div className="p-3 border-t shrink-0">
					<div className="flex gap-2">
						<Input
							placeholder="Type a message..."
							value={newMessage}
							onChange={(e) => setNewMessage(e.target.value)}
							onKeyDown={handleKeyPress}
							className="flex-1 rounded-full bg-muted border-0 focus-visible:ring-1"
						/>
						<Button
							onClick={handleSend}
							size="icon"
							className="rounded-full shrink-0"
							disabled={!newMessage.trim() || isSending}
						>
							<Send className="size-4" />
						</Button>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	);
}
