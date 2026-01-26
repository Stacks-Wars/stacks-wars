"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import { Input } from "@/components/ui/input";
import { CalendarIcon, Clock } from "lucide-react";
import { cn } from "@/lib/utils";

interface DateTimePickerProps {
	date: Date | undefined;
	setDate: (date: Date | undefined) => void;
	disabled?: boolean;
}

export default function DateTimePicker({
	date,
	setDate,
	disabled,
}: DateTimePickerProps) {
	const [isOpen, setIsOpen] = useState(false);
	const [timeValue, setTimeValue] = useState(
		date
			? `${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`
			: "00:00"
	);

	const handleDateSelect = (selectedDate: Date | undefined) => {
		if (!selectedDate) {
			setDate(undefined);
			return;
		}

		// Preserve time if date exists, otherwise use timeValue
		const [hours, minutes] = timeValue.split(":").map(Number);
		const newDate = new Date(selectedDate);
		newDate.setHours(hours || 0);
		newDate.setMinutes(minutes || 0);
		newDate.setSeconds(0);
		setDate(newDate);
	};

	const handleTimeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		const value = e.target.value;
		setTimeValue(value);

		if (date && value) {
			const [hours, minutes] = value.split(":").map(Number);
			const newDate = new Date(date);
			newDate.setHours(hours);
			newDate.setMinutes(minutes);
			newDate.setSeconds(0);
			setDate(newDate);
		}
	};

	const formatDate = (date: Date) => {
		return date.toLocaleDateString("default", {
			month: "short",
			day: "numeric",
			year: "numeric",
		});
	};

	return (
		<div className="flex gap-2">
			{/* Date Picker */}
			<Popover open={isOpen} onOpenChange={setIsOpen}>
				<PopoverTrigger asChild>
					<Button
						variant="outline"
						className={cn(
							"flex-1 justify-start text-left font-normal",
							!date && "text-muted-foreground"
						)}
						disabled={disabled}
					>
						<CalendarIcon className="mr-2 size-4" />
						{date ? formatDate(date) : "Pick a date"}
					</Button>
				</PopoverTrigger>
				<PopoverContent className="w-auto p-0" align="start">
					<Calendar
						mode="single"
						selected={date}
						onSelect={(selectedDate) => {
							handleDateSelect(selectedDate);
							setIsOpen(false);
						}}
						className="roun"
					/>
				</PopoverContent>
			</Popover>

			{/* Time Picker */}
			<div className="relative w-36 flex items-center">
				<Clock className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
				<Input
					type="time"
					value={timeValue}
					onChange={handleTimeChange}
					disabled={disabled}
					className="h-full pl-9 bg-background"
				/>
			</div>
		</div>
	);
}
