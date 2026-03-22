"use client";

import { useState, useEffect, useRef } from "react";
import type { GamePluginProps } from "@/lib/definitions";
import type { LexiWarsState } from "./types";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useUser } from "@/lib/stores/user";
import { cn, displayUserIdentifier } from "@/lib/utils";
import { playSound } from "@/lib/audio/play-sound";
import RoomHeader from "@/components/room/room-header";
import ChatDialog from "@/components/room/chat";
import { toast } from "sonner";

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

		playSound();
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

	const handlePaste = (e: React.ClipboardEvent) => {
		e.preventDefault();
		toast.error("Pasting is not permited!");
	};

	const handleCopy = (e: React.ClipboardEvent) => {
		e.preventDefault();
		toast.error("Copying is not permited!");
	};

	const handleCut = (e: React.ClipboardEvent) => {
		e.preventDefault();
		toast.error("Cutting is not permited!");
	};

	return (
		<>
			<RoomHeader />
			<div className="mx-auto max-w-2xl space-y-6 py-6">
				{/* Game Header */}
				<div className="rounded-lg border p-4">
					<div className="flex items-center justify-between">
						<div>
							<h2 className="text-xl font-bold">{lobby.name}</h2>
							<p className="text-muted-foreground text-sm">
								{game.name}
							</p>
						</div>
						<div className="text-right">
							<div className="text-muted-foreground text-sm">
								Players
							</div>
							<div className="text-lg font-semibold">
								{state.remainingPlayers}/{state.totalPlayers}
							</div>
						</div>
					</div>
				</div>

				{/* Turn Indicator with Timer */}
				<div className="rounded-lg border p-4">
					{state.currentPlayer ? (
						<div className="flex items-center justify-between">
							<div>
								<p className="text-muted-foreground text-sm">
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
					<div className="border-primary bg-primary/5 rounded-lg border p-4">
						<p className="text-primary text-sm font-medium">
							Current Rule
						</p>
						<p className="text-muted-foreground text-sm">
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
						onPaste={handlePaste}
						onCopy={handleCopy}
						onCut={handleCut}
						placeholder={
							isMyTurn
								? "Type your word..."
								: "Waiting for your turn"
						}
						disabled={!isMyTurn || isSubmitting}
						className="text-lg"
						name="no-suggest"
						autoComplete="new-password"
						autoCorrect="off"
						autoCapitalize="off"
						spellCheck={false}
						inputMode="none"
						aria-autocomplete="none"
						autoFocus={isMyTurn || !isSubmitting}
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
			<div className="pointer-events-none fixed right-0 bottom-6 left-0 z-40">
				<div className="container mx-auto flex max-w-2xl justify-end px-4">
					<ChatDialog
						buttonVariant="default"
						buttonClassName="size-12 shadow-lg pointer-events-auto"
					/>
				</div>
			</div>
		</>
	);
}
