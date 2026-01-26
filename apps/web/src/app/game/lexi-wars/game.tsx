"use client";

import { useState, useEffect, useRef } from "react";
import type { GamePluginProps } from "@/lib/definitions";
import type { LexiWarsState } from "./types";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useUser } from "@/lib/stores/user";
import { cn, displayUserIdentifier } from "@/lib/utils";
import RoomHeader from "@/components/room/room-header";
import ChatDialog from "@/components/room/chat";

export default function LexiWarsGame({
	state,
	sendMessage,
	lobby,
	game,
}: GamePluginProps<LexiWarsState>) {
	const [word, setWord] = useState("");
	const [isSubmitting, setIsSubmitting] = useState(false);
	const inputRef = useRef<HTMLInputElement>(null);
	const user = useUser();

	const isMyTurn = state.currentPlayer?.userId === user?.id;

	// Focus input when it becomes our turn
	useEffect(() => {
		if (isMyTurn && inputRef.current) {
			inputRef.current.focus();
		}
	}, [isMyTurn]);

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();

		if (!word.trim() || !isMyTurn || isSubmitting) return;

		setIsSubmitting(true);
		sendMessage("submitWord", { word: word.trim().toLowerCase() });
		setWord("");

		// Reset submitting state after a short delay
		setTimeout(() => setIsSubmitting(false), 500);
	};

	// Timer color based on time remaining
	const timerColor =
		state.timeRemaining <= 3 && isMyTurn
			? "text-red-500"
			: state.timeRemaining <= 5 && isMyTurn
				? "text-yellow-500"
				: "text-primary";

	return (
		<>
			<RoomHeader />
			<div className="max-w-2xl py-6 space-y-6 mx-auto">
				{/* Game Header */}
				<div className="rounded-lg border bg-card p-4">
					<div className="flex items-center justify-between">
						<div>
							<h2 className="text-xl font-bold">{lobby.name}</h2>
							<p className="text-sm text-muted-foreground">
								{game.name}
							</p>
						</div>
						<div className="text-right">
							<div className="text-sm text-muted-foreground">
								Players
							</div>
							<div className="text-lg font-semibold">
								{state.remainingPlayers}/{state.totalPlayers}
							</div>
						</div>
					</div>
				</div>

				{/* Turn Indicator with Timer */}
				<div className="rounded-lg border bg-card p-4">
					{state.currentPlayer ? (
						<div className="flex items-center justify-between">
							<div>
								<p className="text-sm text-muted-foreground">
									{isMyTurn ? "Your Turn!" : "Current Turn"}
								</p>
								<p className="text-lg font-semibold">
									{displayUserIdentifier(state.currentPlayer)}
								</p>
							</div>
							<div
								className={cn(
									"flex h-14 w-14 items-center justify-center rounded-full border-2 text-2xl font-bold transition-colors",
									state.timeRemaining <= 3 && isMyTurn
										? "border-red-500 bg-red-500/10"
										: state.timeRemaining <= 5 && isMyTurn
											? "border-yellow-500 bg-yellow-500/10"
											: "border-primary bg-primary/10"
								)}
							>
								<span className={timerColor}>
									{state.timeRemaining}
								</span>
							</div>
						</div>
					) : (
						<p className="text-muted-foreground text-center">
							Waiting...
						</p>
					)}
				</div>

				{/* Rule Display */}
				{state.currentRule && (
					<div className="rounded-lg border border-primary bg-primary/5 p-4">
						<p className="text-sm font-medium text-primary">
							Current Rule
						</p>
						<p className="text-sm text-muted-foreground">
							{state.currentRule.description}
						</p>
					</div>
				)}

				{/* Word Input */}
				<form onSubmit={handleSubmit} className="space-y-3">
					<Input
						ref={inputRef}
						type="text"
						value={word}
						onChange={(e) => setWord(e.target.value)}
						placeholder={
							isMyTurn
								? "Type your word..."
								: "Waiting for your turn"
						}
						disabled={!isMyTurn || isSubmitting}
						className="text-lg"
						autoComplete="off"
						autoCorrect="off"
						autoCapitalize="off"
						spellCheck={false}
					/>
					<Button
						type="submit"
						disabled={!isMyTurn || !word.trim() || isSubmitting}
						className="w-full"
					>
						{isSubmitting ? "Submitting..." : "Submit Word"}
					</Button>
				</form>
			</div>

			{/* Floating Chat Button - stays within container bounds */}
			<div className="fixed bottom-6 left-0 right-0 z-40 pointer-events-none">
				<div className="container max-w-2xl mx-auto px-4 flex justify-end">
					<ChatDialog
						buttonVariant="default"
						buttonClassName="size-12 shadow-lg pointer-events-auto"
					/>
				</div>
			</div>
		</>
	);
}
