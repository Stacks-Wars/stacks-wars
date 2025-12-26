"use client";

import { useState } from "react";
import { FilterIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { LobbyStatus } from "@/lib/definitions";

interface LobbyFilterProps {
	value: LobbyStatus[];
	onChange: (statuses: LobbyStatus[]) => void;
}

const statusOptions: { value: LobbyStatus; label: string }[] = [
	{ value: "waiting", label: "Waiting" },
	{ value: "starting", label: "Starting" },
	{ value: "inProgress", label: "In Progress" },
	{ value: "finished", label: "Finished" },
];

export function LobbyFilter({ value, onChange }: LobbyFilterProps) {
	const [selectedStatuses, setSelectedStatuses] =
		useState<LobbyStatus[]>(value);

	const handleToggle = (status: LobbyStatus, checked: boolean) => {
		const newStatuses = checked
			? [...selectedStatuses, status]
			: selectedStatuses.filter((s) => s !== status);

		setSelectedStatuses(newStatuses);
		onChange(newStatuses);
	};

	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button
					variant="outline"
					className="rounded-full p-2.5 sm:px-7 sm:py-3.5 has-[>svg]:px-2.5 sm:has-[>svg]:px-7 text-sm sm:text-base"
				>
					<FilterIcon className="size-3.5 sm:size-4" />
					Filter Lobbies
					{selectedStatuses.length > 0 && (
						<span className="rounded-full bg-primary px-2 py-1 text-xs text-primary-foreground">
							{selectedStatuses.length}
						</span>
					)}
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end" className="w-56">
				<DropdownMenuLabel>Filter by Status</DropdownMenuLabel>
				<DropdownMenuSeparator />
				<div className="space-y-2 p-2">
					{statusOptions.map((option) => {
						const isChecked = selectedStatuses.includes(
							option.value
						);
						return (
							<div
								key={option.value}
								className="flex items-center gap-2"
							>
								<Checkbox
									id={option.value}
									checked={isChecked}
									onCheckedChange={(checked) =>
										handleToggle(
											option.value,
											checked as boolean
										)
									}
								/>
								<label
									htmlFor={option.value}
									className="text-sm cursor-pointer flex-1"
								>
									{option.label}
								</label>
							</div>
						);
					})}
				</div>
			</DropdownMenuContent>
		</DropdownMenu>
	);
}
