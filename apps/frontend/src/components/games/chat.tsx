"use client";

import { useState, useRef, useEffect } from "react";
import {
	Dialog,
	DialogContent,
	DialogTrigger,
	DialogHeader,
	DialogTitle,
	DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { SendHorizonal, Users, Wifi, WifiOff } from "lucide-react";
import { cn, truncateAddress } from "@/lib/utils";
import { ChatMessage } from "@/hooks/useChatSocket";
import { useChatSocketContext } from "@/contexts/ChatSocketProvider";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import { FiMessageCircle } from "react-icons/fi";

export default function Chat() {
	const [open, setOpen] = useState(false);
	const [input, setInput] = useState("");
	const inputRef = useRef<HTMLInputElement>(null);

	const {
		sendMessage,
		readyState,
		reconnecting,
		messages,
		unreadCount,
		chatPermitted,
		setOpen: setContextOpen,
		userId,
	} = useChatSocketContext();

	useEffect(() => {
		setContextOpen(open);
	}, [open, setContextOpen]);

	const viewportRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		const el = viewportRef.current;
		if (el) {
			el.scrollTop = el.scrollHeight;
		}
	}, [messages]);

	// Focus input when dialog opens
	useEffect(() => {
		if (open && inputRef.current) {
			inputRef.current.focus();
		}
	}, [open]);

	const handleSend = async () => {
		if (!input.trim() || !chatPermitted) return;

		try {
			await sendMessage({ type: "chat", text: input.trim() });
			setInput("");
		} catch (error) {
			console.error("Failed to send chat message:", error);
		}
	};

	const formatTimestamp = (timestamp: string) => {
		return new Date(timestamp).toLocaleTimeString([], {
			hour: "2-digit",
			minute: "2-digit",
		});
	};

	const getDisplayName = (sender: ChatMessage["sender"]) => {
		return (
			sender.user.displayName ||
			sender.user.username ||
			truncateAddress(sender.user.walletAddress)
		);
	};

	const getInitials = (name: string) => {
		return name
			.split(" ")
			.map((part) => part[0])
			.join("")
			.toUpperCase()
			.slice(0, 2);
	};

	return (
		<div className="pointer-events-none fixed inset-x-0 bottom-0 z-50 mx-auto max-w-7xl">
			<div className="relative w-full">
				<Dialog open={open} onOpenChange={setOpen}>
					<DialogTrigger asChild>
						<Button
							variant="outline"
							className="from-primary/10 to-primary/20 border-primary/20 hover:from-primary/20 hover:to-primary/30 group pointer-events-auto absolute right-6 bottom-6 size-12 rounded-full bg-gradient-to-r p-4 shadow-lg backdrop-blur-sm"
						>
							<div className="relative">
								<FiMessageCircle className="text-primary size-6 rounded-full transition-transform group-hover:scale-110" />
								{unreadCount > 0 && (
									<Badge
										variant="destructive"
										className="absolute -top-2 -right-2 flex h-6 w-6 animate-pulse items-center justify-center rounded-full p-0 text-xs font-semibold"
									>
										{unreadCount > 99 ? "99+" : unreadCount}
									</Badge>
								)}
							</div>
							<span className="sr-only">
								Open chat{" "}
								{unreadCount > 0 && `(${unreadCount} unread)`}
							</span>
						</Button>
					</DialogTrigger>

					<DialogContent
						className={cn(
							"border-primary/30 max-h-[85vh] w-full gap-0 overflow-hidden rounded-xl p-0 transition-all duration-300 sm:max-w-lg"
						)}
						hideClose
					>
						<DialogHeader className="from-primary to-primary/80 text-primary-foreground border-primary/30 border-b bg-gradient-to-r px-6 py-4">
							<div className="flex items-center justify-between">
								<div className="flex items-center gap-3">
									<Users className="h-5 w-5" />
									<div>
										<DialogTitle className="text-lg font-semibold">
											Game Chat
										</DialogTitle>
										<DialogDescription className="text-primary-foreground/80 text-sm">
											{messages.length} messages •{" "}
											{
												new Set(
													messages.map(
														(m) => m.sender.id
													)
												).size
											}{" "}
											players
										</DialogDescription>
									</div>
								</div>
								<Tooltip>
									<TooltipTrigger asChild>
										<div className="flex items-center gap-1 text-sm">
											{readyState === WebSocket.OPEN ? (
												<Wifi className="h-4 w-4 text-green-300" />
											) : (
												<WifiOff className="h-4 w-4 text-amber-300" />
											)}
											<span>
												{readyState === WebSocket.OPEN
													? "Connected"
													: reconnecting
														? "Reconnecting..."
														: "Disconnected"}
											</span>
										</div>
									</TooltipTrigger>
									<TooltipContent>
										{readyState === WebSocket.OPEN
											? "Connection is active"
											: reconnecting
												? "Attempting to reconnect..."
												: "Chat is disconnected"}
									</TooltipContent>
								</Tooltip>
							</div>
						</DialogHeader>

						<div className="bg-background/95 flex h-full flex-col backdrop-blur-sm">
							<ScrollArea
								className={cn("h-[400px] w-full px-4 py-3")}
								ref={viewportRef}
							>
								{!chatPermitted ? (
									<div
										className={cn(
											"flex h-full min-h-72 flex-col items-center justify-center gap-4"
										)}
									>
										<div className="bg-primary/10 rounded-full p-6">
											<FiMessageCircle className="text-primary h-12 w-12" />
										</div>
										<div className="space-y-1 text-center">
											<h3 className="text-lg font-medium">
												Chat is not availble
											</h3>
											<p className="text-muted-foreground max-w-md">
												You need to join the lobby to
												participate in the chat.
											</p>
										</div>
									</div>
								) : messages.length === 0 ? (
									<div
										className={cn(
											"flex h-full min-h-72 flex-col items-center justify-center gap-4"
										)}
									>
										<div className="bg-primary/10 rounded-full p-6">
											<FiMessageCircle className="text-primary h-12 w-12" />
										</div>
										<div className="space-y-1 text-center">
											<h3 className="text-lg font-medium">
												No messages yet
											</h3>
											<p className="text-muted-foreground">
												Be the first to start the
												conversation!
											</p>
										</div>
									</div>
								) : (
									<div className="space-y-3 pb-2">
										{messages.map((msg, index) => {
											const isOwnMessage =
												msg.sender.id === userId;
											const showSenderInfo =
												index === 0 ||
												messages[index - 1].sender
													.id !== msg.sender.id;

											const isLastFromSender =
												index === messages.length - 1 ||
												messages[index + 1].sender
													.id !== msg.sender.id;

											return (
												<div
													key={msg.id}
													className={cn(
														"flex gap-2",
														isOwnMessage
															? "justify-end"
															: "justify-start"
													)}
												>
													{!isOwnMessage && (
														<div className="flex-shrink-0">
															{showSenderInfo ? (
																<Tooltip>
																	<TooltipTrigger
																		asChild
																	>
																		<Avatar className="h-8 w-8">
																			<AvatarImage
																				src={
																					""
																				}
																				alt={getDisplayName(
																					msg.sender
																				)}
																			/>
																			<AvatarFallback className="bg-primary/10 text-primary">
																				{getInitials(
																					getDisplayName(
																						msg.sender
																					)
																				)}
																			</AvatarFallback>
																		</Avatar>
																	</TooltipTrigger>
																	<TooltipContent>
																		{getDisplayName(
																			msg.sender
																		)}
																	</TooltipContent>
																</Tooltip>
															) : (
																<div className="h-8 w-8" /> // Spacer for alignment
															)}
														</div>
													)}

													<div
														className={cn(
															"flex flex-col",
															isOwnMessage
																? "items-end"
																: "items-start",
															"max-w-[75%] min-w-0"
														)}
													>
														{showSenderInfo &&
															!isOwnMessage && (
																<span className="text-foreground mb-1 text-xs font-medium">
																	{getDisplayName(
																		msg.sender
																	)}
																</span>
															)}

														<div
															className={cn(
																"rounded-xl px-4 py-2 text-sm shadow-sm",
																"overflow-wrap-anywhere break-words break-all hyphens-auto whitespace-pre-wrap",
																"max-w-full",
																isOwnMessage
																	? "bg-primary text-primary-foreground rounded-br-none"
																	: "bg-muted text-foreground border-muted-foreground/10 rounded-bl-none border"
															)}
														>
															{msg.text}
														</div>

														{isLastFromSender && (
															<span
																className={cn(
																	"text-muted-foreground mt-1 px-1 text-xs",
																	isOwnMessage
																		? "text-right"
																		: "text-left"
																)}
															>
																{formatTimestamp(
																	msg.timestamp
																)}
															</span>
														)}
													</div>
												</div>
											);
										})}
									</div>
								)}
							</ScrollArea>

							{chatPermitted && (
								<div
									className={cn(
										"border-primary/20 bg-background border-t p-4"
									)}
								>
									<form
										onSubmit={(e) => {
											e.preventDefault();
											handleSend();
										}}
										className="flex items-end gap-3"
									>
										<div className="relative min-w-0 flex-1">
											<Input
												ref={inputRef}
												value={input}
												onChange={(e) =>
													setInput(e.target.value)
												}
												placeholder={
													readyState ===
													WebSocket.OPEN
														? "Type your message..."
														: "Connecting to chat..."
												}
												disabled={
													readyState !==
													WebSocket.OPEN
												}
												className="bg-background border-primary/30 focus:border-primary/50 focus:ring-primary/20 pr-12 break-all"
												maxLength={500}
												onKeyDown={(e) => {
													if (
														e.key === "Enter" &&
														!e.shiftKey
													) {
														e.preventDefault();
														handleSend();
													}
												}}
											/>
											<div className="text-muted-foreground absolute top-1/2 right-3 -translate-y-1/2 text-xs">
												{input.length}/500
											</div>
										</div>
										<Tooltip>
											<TooltipTrigger asChild>
												<Button
													type="submit"
													size="icon"
													disabled={
														!input.trim() ||
														readyState !==
															WebSocket.OPEN
													}
													className="bg-primary hover:bg-primary/90 h-10 w-10 shrink-0"
												>
													<SendHorizonal className="h-4 w-4" />
													<span className="sr-only">
														Send message
													</span>
												</Button>
											</TooltipTrigger>
											<TooltipContent>
												{readyState !== WebSocket.OPEN
													? "Connecting..."
													: !input.trim()
														? "Enter a message"
														: "Send message"}
											</TooltipContent>
										</Tooltip>
									</form>
								</div>
							)}
						</div>
					</DialogContent>
				</Dialog>
			</div>
		</div>
	);
}
