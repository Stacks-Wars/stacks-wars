"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Loader2 } from "lucide-react";

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
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { ApiClient } from "@/lib/api/client";
import { useRouter } from "next/navigation";
import type { CreateGameRequest, Game } from "@/lib/definitions";

// Game categories
const GAME_CATEGORIES = [
	"Word Games",
	"Strategy",
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
		category: z.string().min(1, "Category is required"),
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
						"Must be a valid URL or relative path (e.g., /images/game.svg)",
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
	const router = useRouter();

	const form = useForm<CreateGameFormValues>({
		resolver: zodResolver(createGameSchema),
		defaultValues: {
			name: "",
			description: "",
			imageUrl: "",
			minPlayers: "2",
			maxPlayers: "10",
			category: "",
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

			// Success - redirect to game page
			if (response.data) {
				router.push(`/game/${response.data.path}`);
				onSuccess?.();
			}
		} catch (err) {
			const errorMessage =
				err instanceof Error ? err.message : "Failed to create game";
			setError(errorMessage);
		}
	};

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
									className="text-sm sm:text-base h-10 sm:h-12"
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
									placeholder="/images/game.svg or https://..."
									className="text-sm sm:text-base h-10 sm:h-12"
									{...field}
								/>
							</FormControl>
							<FormDescription className="text-xs sm:text-sm">
								Provide a valid URL or relative path
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>

				<div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
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
										className="text-sm sm:text-base h-10 sm:h-12"
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
										className="text-sm sm:text-base h-10 sm:h-12"
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
					render={({ field }) => (
						<FormItem>
							<FormLabel className="text-sm sm:text-base">
								Category
							</FormLabel>
							<Select
								onValueChange={field.onChange}
								defaultValue={field.value}
							>
								<FormControl>
									<SelectTrigger className="text-sm sm:text-base h-10 sm:h-12 w-full">
										<SelectValue placeholder="Select a category" />
									</SelectTrigger>
								</FormControl>
								<SelectContent>
									{GAME_CATEGORIES.map((category) => (
										<SelectItem
											key={category}
											value={category}
											className="text-sm sm:text-base"
										>
											{category}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
							<FormDescription className="text-xs sm:text-sm">
								Choose the game category
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>

				{error && (
					<div className="text-sm text-destructive">{error}</div>
				)}

				<div className="flex justify-end gap-3 pt-2">
					<Button
						type="submit"
						className="rounded-full text-sm sm:text-base has-[>svg]:px-8 w-full sm:w-auto"
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
