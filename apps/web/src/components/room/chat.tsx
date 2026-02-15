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
	useLobby,
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
	const lobby = useLobby();
	const { sendLobbyMessage } = useRoom();
	const scrollRef = useRef<HTMLDivElement>(null);
	const [showReactionPicker, setShowReactionPicker] = useState<string | null>(
		null
	);

	const [newMessage, setNewMessage] = useState("");
	const isSending = useIsActionLoading("sendMessage");
	const isFinished = lobby?.status === "finished";

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
								<span className="bg-primary text-primary-foreground absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full text-[10px] font-medium">
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
					"flex h-[70vh] max-h-150 max-w-md flex-col gap-0 p-0",
					className
				)}
			>
				<DialogHeader className="shrink-0 border-b px-4 py-3">
					<DialogTitle className="flex items-center gap-2">
						<MessageCircle className="size-5" />
						Lobby Chat
						<span className="text-muted-foreground text-xs font-normal">
							({players.length} online)
						</span>
					</DialogTitle>
				</DialogHeader>

				{/* Messages Area */}
				<ScrollArea ref={scrollRef} className="flex-1 px-4">
					<div className="space-y-3 py-4">
						{messages.length === 0 ? (
							<div className="flex flex-col items-center justify-center py-12 text-center">
								<MessageCircle className="text-muted-foreground/30 mb-3 size-12" />
								<p className="text-muted-foreground text-sm">
									No messages yet
								</p>
								<p className="text-muted-foreground/70 text-xs">
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
											<span className="text-muted-foreground text-xs font-medium">
												{isOwn
													? "You"
													: getDisplayName(
															msg.userId
														)}
											</span>
											<span className="text-muted-foreground/60 text-[10px]">
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
												"relative w-fit max-w-[85%] rounded-2xl px-3 py-2 text-sm",
												isOwn
													? "bg-primary text-primary-foreground rounded-br-md"
													: "bg-muted rounded-bl-md"
											)}
										>
											{msg.content}

											{/* Reaction button (shows on hover, hidden when finished) */}
											{!isFinished && (
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
														"absolute -bottom-2 opacity-0 transition-opacity group-hover:opacity-100",
														"bg-background flex h-6 w-6 items-center justify-center rounded-full border shadow-sm",
														"hover:bg-muted",
														isOwn
															? "left-0 -translate-x-1/2"
															: "right-0 translate-x-1/2"
													)}
												>
													<Smile className="text-muted-foreground size-3" />
												</button>
											)}

											{/* Quick reaction picker */}
											{!isFinished &&
												showReactionPicker ===
													msg.messageId && (
													<div
														className={cn(
															"bg-background absolute -bottom-9 z-10 flex gap-1 rounded-full border px-2 py-1 shadow-lg",
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
																	className="text-base transition-transform hover:scale-125"
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
													"mt-1 flex flex-wrap gap-1",
													isOwn && "justify-end"
												)}
											>
												{Object.entries(
													groupedReactions
												).map(([emoji, userIds]) => (
													<button
														key={emoji}
														className={cn(
															"rounded-full border px-2 py-0.5 text-xs transition-colors",
															isFinished &&
																"pointer-events-none opacity-60",
															user?.id &&
																userIds.includes(
																	user.id
																)
																? "bg-primary/10 border-primary/30"
																: "bg-muted/50 hover:border-muted-foreground/20 border-transparent"
														)}
														disabled={isFinished}
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
				<div className="shrink-0 border-t p-3">
					<div className="flex gap-2">
						<Input
							placeholder={
								isFinished
									? "Chat is closed"
									: "Type a message..."
							}
							value={newMessage}
							onChange={(e) => setNewMessage(e.target.value)}
							onKeyDown={handleKeyPress}
							disabled={isFinished}
							className="bg-muted flex-1 rounded-full border-0 focus-visible:ring-1"
						/>
						<Button
							onClick={handleSend}
							size="icon"
							className="shrink-0 rounded-full"
							disabled={
								!newMessage.trim() || isSending || isFinished
							}
						>
							<Send className="size-4" />
						</Button>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	);
}
