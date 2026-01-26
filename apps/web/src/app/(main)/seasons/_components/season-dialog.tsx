"use client";

import { useState } from "react";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import type { Season } from "@/lib/definitions";
import { ApiClient } from "@/lib/api/client";
import { toast } from "sonner";
import DateTimePicker from "./date-time-picker";

interface SeasonDialogProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	season?: Season;
	onSuccess: (season: Season) => void;
}

export default function SeasonDialog({
	open,
	onOpenChange,
	season,
	onSuccess,
}: SeasonDialogProps) {
	const isEditing = !!season;
	const [isSubmitting, setIsSubmitting] = useState(false);

	// Form state
	const [name, setName] = useState(season?.name || "");
	const [description, setDescription] = useState(season?.description || "");
	const [startDate, setStartDate] = useState<Date | undefined>(
		season?.startDate ? new Date(season.startDate) : undefined
	);
	const [endDate, setEndDate] = useState<Date | undefined>(
		season?.endDate ? new Date(season.endDate) : undefined
	);

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();

		if (!name.trim()) {
			toast.error("Season name is required");
			return;
		}

		if (!startDate || !endDate) {
			toast.error("Start and end dates are required");
			return;
		}

		if (endDate <= startDate) {
			toast.error("End date must be after start date");
			return;
		}

		setIsSubmitting(true);

		// Format dates to "YYYY-MM-DD HH:MM:SS"
		const formatDate = (date: Date) => {
			const year = date.getFullYear();
			const month = String(date.getMonth() + 1).padStart(2, "0");
			const day = String(date.getDate()).padStart(2, "0");
			const hours = String(date.getHours()).padStart(2, "0");
			const minutes = String(date.getMinutes()).padStart(2, "0");
			const seconds = String(date.getSeconds()).padStart(2, "0");
			return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
		};

		const payload = {
			name: name.trim(),
			description: description.trim() || undefined,
			startDate: formatDate(startDate),
			endDate: formatDate(endDate),
		};

		try {
			if (isEditing) {
				const response = await ApiClient.put<Season>(
					`/api/season/${season.id}`,
					payload
				);

				if (response.error || !response.data) {
					toast.error("Failed to update season", {
						description: response.error || "Unknown error",
					});
					return;
				}

				toast.success("Season updated successfully");
				onSuccess(response.data);
			} else {
				const response = await ApiClient.post<Season>(
					"/api/season",
					payload
				);

				if (response.error || !response.data) {
					toast.error("Failed to create season", {
						description: response.error || "Unknown error",
					});
					return;
				}

				toast.success("Season created successfully");
				onSuccess(response.data);
				// Reset form
				setName("");
				setDescription("");
				setStartDate(undefined);
				setEndDate(undefined);
			}
		} catch (error) {
			toast.error("An unexpected error occurred");
		} finally {
			setIsSubmitting(false);
		}
	};

	const handleOpenChange = (open: boolean) => {
		if (!open && !isSubmitting) {
			// Reset form when closing if creating
			if (!isEditing) {
				setName("");
				setDescription("");
				setStartDate(undefined);
				setEndDate(undefined);
			}
		}
		onOpenChange(open);
	};

	return (
		<Dialog open={open} onOpenChange={handleOpenChange}>
			<DialogContent className="sm:max-w-lg">
				<DialogHeader>
					<DialogTitle>
						{isEditing ? "Edit Season" : "Create New Season"}
					</DialogTitle>
					<DialogDescription>
						{isEditing
							? "Update season details and dates"
							: "Create a new competitive season for leaderboard tracking"}
					</DialogDescription>
				</DialogHeader>

				<form onSubmit={handleSubmit} className="space-y-4">
					{/* Name */}
					<div className="space-y-2">
						<Label htmlFor="name">Season Name</Label>
						<Input
							id="name"
							placeholder="e.g., Season 1: Winter Wars"
							value={name}
							onChange={(e) => setName(e.target.value)}
							disabled={isSubmitting}
							required
						/>
					</div>

					{/* Description */}
					<div className="space-y-2">
						<Label htmlFor="description">
							Description (Optional)
						</Label>
						<Textarea
							id="description"
							placeholder="Brief description of the season..."
							value={description}
							onChange={(e) => setDescription(e.target.value)}
							disabled={isSubmitting}
							rows={3}
						/>
					</div>

					{/* Start Date */}
					<div className="space-y-2">
						<Label>Start Date & Time</Label>
						<DateTimePicker
							date={startDate}
							setDate={setStartDate}
							disabled={isSubmitting}
						/>
					</div>

					{/* End Date */}
					<div className="space-y-2">
						<Label>End Date & Time</Label>
						<DateTimePicker
							date={endDate}
							setDate={setEndDate}
							disabled={isSubmitting}
						/>
					</div>

					{/* Actions */}
					<div className="flex justify-end gap-3 pt-4">
						<Button
							type="button"
							variant="outline"
							onClick={() => handleOpenChange(false)}
							disabled={isSubmitting}
						>
							Cancel
						</Button>
						<Button type="submit" disabled={isSubmitting}>
							{isSubmitting
								? isEditing
									? "Updating..."
									: "Creating..."
								: isEditing
									? "Update Season"
									: "Create Season"}
						</Button>
					</div>
				</form>
			</DialogContent>
		</Dialog>
	);
}
