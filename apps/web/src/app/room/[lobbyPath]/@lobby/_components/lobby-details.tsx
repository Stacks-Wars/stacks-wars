"use client";

import type { Game, LobbyExtended } from "@/lib/definitions";
import { Button } from "@/components/ui/button";
import { MessageCircleMore } from "lucide-react";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { useState } from "react";

export default function LobbyDetails({ lobby, game }: { lobby: LobbyExtended; game: Game }) {
	const [chatOpen, setChatOpen] = useState(false);

	return (
		<div className="flex flex-col items-center w-full">
			<div className="bg-card border rounded-3xl p-4 sm:p-6 lg:p-8 space-y-4 sm:space-y-6 w-full">
				<div className="space-y-2">
					<h2 className="text-lg sm:text-xl lg:text-2xl font-semibold">
						{lobby.name}
					</h2>
					{lobby.description && (
						<p className="text-xs sm:text-sm">
							{lobby.description}
						</p>
					)}
				</div>

				<div className="grid grid-cols-2 md:grid-cols-4 gap-3 sm:gap-4">
					{lobby.entryAmount !== undefined && (
						<div className="space-y-0.5 sm:space-y-1">
							<p className="text-xs sm:text-sm text-muted-foreground">
								Entry Amount
							</p>
							<p className="text-base sm:text-lg lg:text-xl font-medium truncate">
								{lobby.entryAmount} {lobby.tokenSymbol || "STX"}
							</p>
						</div>
					)}

					{lobby.currentAmount !== undefined && (
						<div className="space-y-0.5 sm:space-y-1">
							<p className="text-xs sm:text-sm text-muted-foreground">
								Prize Pool
							</p>
							<p className="text-base sm:text-lg lg:text-xl font-medium truncate">
								{lobby.currentAmount}{" "}
								{lobby.tokenSymbol || "STX"}
							</p>
						</div>
					)}

					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-xs sm:text-sm text-muted-foreground">
							Players
						</p>
						<p className="text-base sm:text-lg lg:text-xl font-medium">
							{lobby.participantCount}/{game.maxPlayers}
						</p>
					</div>

					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-xs sm:text-sm text-muted-foreground">
							Status
						</p>
						<p className="text-base sm:text-lg lg:text-xl font-medium capitalize">
							{lobby.status === "inProgress"
								? "In Progress"
								: lobby.status}
						</p>
					</div>
				</div>
			</div>
			<Dialog open={chatOpen} onOpenChange={setChatOpen}>
				<DialogTrigger asChild>
					<Button
						variant="default"
						size="icon"
						className="-translate-y-1/2 rounded-full size-10 sm:size-12 lg:size-16 -mb-5 sm:-mb-6 lg:-mb-8"
					>
						<MessageCircleMore className="size-4 sm:size-5 lg:size-7" />
					</Button>
				</DialogTrigger>
				<DialogContent className="max-w-[95vw] sm:max-w-2xl">
					<DialogHeader>
						<DialogTitle className="text-lg sm:text-xl">
							Lobby Chat
						</DialogTitle>
						<DialogDescription>
							Chat with other players in this lobby
						</DialogDescription>
					</DialogHeader>
					<div className="min-h-96">
						{/* Chat content will go here */}
						<p className="text-center text-muted-foreground py-20">
							Send a message to other players in this lobby.
						</p>
					</div>
				</DialogContent>
			</Dialog>
		</div>
	);
}
