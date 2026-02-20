"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Loader2, Copy, Check } from "lucide-react";

import {
	Form,
	FormControl,
	FormDescription,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import { ApiClient } from "@/lib/api/client";
import { useRouter } from "next/navigation";
import type { CreateGameRequest, Game } from "@/lib/definitions";

// Game categories
const GAME_CATEGORIES = [
	"Word Games",
	"Strategy",
	"Competitive",
	"Trivia",
	"Card Games",
	"Puzzle",
	"Action",
	"Casual",
] as const;

// Validation schema
const createGameSchema = z
	.object({
		name: z
			.string()
			.min(1, "Game name is required")
			.max(50, "Game name must be at most 50 characters"),
		description: z
			.string()
			.min(1, "Description is required")
			.max(500, "Description must be at most 500 characters"),
		imageUrl: z.string().min(1, "Image URL is required"),
		minPlayers: z
			.string()
			.min(1, "Minimum players is required")
			.max(50, "Minimum players cannot exceed 50"),
		maxPlayers: z
			.string()
			.min(1, "Maximum players is required")
			.max(100, "Maximum players cannot exceed 100"),
		category: z
			.array(z.string())
			.min(1, "At least one category is required")
			.max(3, "Maximum 3 categories allowed"),
	})
	.superRefine((data, ctx) => {
		// Validate imageUrl
		if (!data.imageUrl.startsWith("/")) {
			try {
				new URL(data.imageUrl);
			} catch {
				ctx.addIssue({
					code: "custom",
					message:
						"Must be a valid URL or relative path (e.g., /games/game.png)",
					path: ["imageUrl"],
				});
			}
		}

		// Validate maxPlayers >= minPlayers
		const minNum = parseInt(data.minPlayers);
		const maxNum = parseInt(data.maxPlayers);
		if (!isNaN(minNum) && !isNaN(maxNum) && maxNum < minNum) {
			ctx.addIssue({
				code: "custom",
				message:
					"Maximum players must be greater than or equal to minimum players",
				path: ["maxPlayers"],
			});
		}
	});

type CreateGameFormValues = z.infer<typeof createGameSchema>;

interface CreateGameFormProps {
	onSuccess?: () => void;
}

export default function CreateGameForm({ onSuccess }: CreateGameFormProps) {
	const [error, setError] = useState<string | null>(null);
	const [createdGame, setCreatedGame] = useState<Game | null>(null);
	const [copied, setCopied] = useState(false);
	const router = useRouter();

	const form = useForm<CreateGameFormValues>({
		// @ts-ignore - Zod v4 compatibility issue with @hookform/resolvers
		resolver: zodResolver(createGameSchema),
		defaultValues: {
			name: "",
			description: "",
			imageUrl: "",
			minPlayers: "2",
			maxPlayers: "10",
			category: [],
		},
	});

	const onSubmit = async (values: CreateGameFormValues) => {
		setError(null);

		try {
			// Generate path from name
			const path = values.name.toLowerCase().replace(/\s+/g, "-");

			const payload: CreateGameRequest = {
				name: values.name,
				path,
				description: values.description,
				imageUrl: values.imageUrl,
				minPlayers: parseInt(values.minPlayers),
				maxPlayers: parseInt(values.maxPlayers),
				category: values.category,
			};

			const response = await ApiClient.post<Game>("/api/game", payload);

			if (response.error) {
				setError(response.error);
				return;
			}

			// Success — show the game UUID so the user can copy it
			if (response.data) {
				setCreatedGame(response.data);
				onSuccess?.();
			}
		} catch (err) {
			const errorMessage =
				err instanceof Error ? err.message : "Failed to create game";
			setError(errorMessage);
		}
	};

	const copyGameId = async () => {
		if (!createdGame) return;
		await navigator.clipboard.writeText(createdGame.id);
		setCopied(true);
		setTimeout(() => setCopied(false), 2000);
	};

	if (createdGame) {
		return (
			<div className="space-y-6 text-center">
				<div className="space-y-2">
					<div className="text-3xl">🎮</div>
					<h3 className="text-xl font-bold">Game Created!</h3>
					<p className="text-muted-foreground text-sm">
						<strong>{createdGame.name}</strong> has been created
						successfully.
					</p>
				</div>

				<div className="bg-muted/50 rounded-xl p-4">
					<p className="text-muted-foreground mb-2 text-xs font-medium tracking-wider uppercase">
						Game UUID
					</p>
					<div className="flex items-center justify-center gap-2">
						<code className="bg-background rounded-lg px-3 py-2 font-mono text-sm">
							{createdGame.id}
						</code>
						<Button
							variant="outline"
							size="icon"
							className="h-9 w-9 shrink-0"
							onClick={copyGameId}
						>
							{copied ? (
								<Check className="h-4 w-4 text-green-500" />
							) : (
								<Copy className="h-4 w-4" />
							)}
						</Button>
					</div>
					<p className="text-muted-foreground mt-3 text-xs">
						Copy and save this UUID — you&apos;ll need it for your
						game registry in{" "}
						<code className="text-xs">registry.rs</code>
					</p>
				</div>

				<div className="flex justify-center gap-3">
					<Button
						className="rounded-full"
						onClick={() => router.push(`/game/${createdGame.path}`)}
					>
						View Game Page
					</Button>
				</div>
			</div>
		);
	}

	return (
		<Form {...form}>
			<form
				onSubmit={form.handleSubmit(onSubmit)}
				className="space-y-4 sm:space-y-6"
			>
				<FormField
					control={form.control}
					name="name"
					render={({ field }) => (
						<FormItem>
							<FormLabel className="text-sm sm:text-base">
								Game Name
							</FormLabel>
							<FormControl>
								<Input
									placeholder="Enter game name"
									className="h-10 text-sm sm:h-12 sm:text-base"
									{...field}
									maxLength={50}
								/>
							</FormControl>
							<FormDescription className="text-xs sm:text-sm">
								Maximum 50 characters
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>

				<FormField
					control={form.control}
					name="description"
					render={({ field }) => (
						<FormItem>
							<FormLabel className="text-sm sm:text-base">
								Description
							</FormLabel>
							<FormControl>
								<Textarea
									placeholder="Describe your game"
									{...field}
									maxLength={500}
									rows={3}
								/>
							</FormControl>
							<FormDescription className="text-xs sm:text-sm">
								Maximum 500 characters
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>

				<FormField
					control={form.control}
					name="imageUrl"
					render={({ field }) => (
						<FormItem>
							<FormLabel className="text-sm sm:text-base">
								Image URL
							</FormLabel>
							<FormControl>
								<Input
									placeholder="/games/game.png or https://..."
									className="h-10 text-sm sm:h-12 sm:text-base"
									{...field}
								/>
							</FormControl>
							<FormDescription className="text-xs sm:text-sm">
								Provide a valid URL or relative path in stacks
								wars public folder
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>

				<div className="grid grid-cols-1 gap-4 sm:grid-cols-2 sm:gap-6">
					<FormField
						control={form.control}
						name="minPlayers"
						render={({ field }) => (
							<FormItem>
								<FormLabel className="text-sm sm:text-base">
									Min Players
								</FormLabel>
								<FormControl>
									<Input
										type="number"
										placeholder="2"
										className="h-10 text-sm sm:h-12 sm:text-base"
										{...field}
										min={2}
										max={16}
									/>
								</FormControl>
								<FormDescription className="text-xs sm:text-sm">
									2-16 players
								</FormDescription>
								<FormMessage />
							</FormItem>
						)}
					/>

					<FormField
						control={form.control}
						name="maxPlayers"
						render={({ field }) => (
							<FormItem>
								<FormLabel className="text-sm sm:text-base">
									Max Players
								</FormLabel>
								<FormControl>
									<Input
										type="number"
										placeholder="10"
										className="h-10 text-sm sm:h-12 sm:text-base"
										{...field}
										min={2}
										max={100}
									/>
								</FormControl>
								<FormDescription className="text-xs sm:text-sm">
									Up to 100 players
								</FormDescription>
								<FormMessage />
							</FormItem>
						)}
					/>
				</div>

				<FormField
					control={form.control}
					name="category"
					render={() => (
						<FormItem>
							<FormLabel className="text-sm sm:text-base">
								Categories
							</FormLabel>
							<div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
								{GAME_CATEGORIES.map((category) => (
									<FormField
										key={category}
										control={form.control}
										name="category"
										render={({ field }) => {
											return (
												<FormItem
													key={category}
													className="flex flex-row items-start space-y-0 space-x-3"
												>
													<FormControl>
														<Checkbox
															checked={field.value?.includes(
																category
															)}
															onCheckedChange={(
																checked
															) => {
																return checked
																	? field.onChange(
																			[
																				...field.value,
																				category,
																			]
																		)
																	: field.onChange(
																			field.value?.filter(
																				(
																					value
																				) =>
																					value !==
																					category
																			)
																		);
															}}
														/>
													</FormControl>
													<FormLabel className="cursor-pointer text-sm font-normal">
														{category}
													</FormLabel>
												</FormItem>
											);
										}}
									/>
								))}
							</div>
							<FormDescription className="text-xs sm:text-sm">
								Select 1-3 categories for your game
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>

				{error && (
					<div className="text-destructive text-sm">{error}</div>
				)}

				<div className="flex justify-end gap-3 pt-2">
					<Button
						type="submit"
						className="w-full rounded-full text-sm has-[>svg]:px-8 sm:w-auto sm:text-base"
						disabled={form.formState.isSubmitting}
					>
						{form.formState.isSubmitting ? (
							<>
								<Loader2 className="mr-2 h-4 w-4 animate-spin" />
								Creating...
							</>
						) : (
							"Create Game"
						)}
					</Button>
				</div>
			</form>
		</Form>
	);
}
