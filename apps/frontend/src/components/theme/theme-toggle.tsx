"use client";
import { Moon, Sun, Laptop } from "lucide-react";
import { useTheme } from "next-themes";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { HiMiniComputerDesktop } from "react-icons/hi2";

export function ThemeToggle({ className }: { className?: string }) {
	const { setTheme, theme } = useTheme();

	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button variant="ghost" className={className}>
					{theme === "system" ? (
						<HiMiniComputerDesktop className="size-[1.2rem]" />
					) : theme === "dark" ? (
						<Moon className="size-[1.2rem]" />
					) : (
						<Sun className="size-[1.2rem]" />
					)}
					<span className="capitalize">{theme}</span>
					<span className="sr-only">Toggle theme</span>
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end">
				<DropdownMenuItem
					onClick={() => setTheme("light")}
					className="flex items-center space-x-2"
				>
					<Sun className="h-4 w-4" />
					<span>Light</span>
				</DropdownMenuItem>
				<DropdownMenuItem
					onClick={() => setTheme("dark")}
					className="flex items-center space-x-2"
				>
					<Moon className="h-4 w-4" />
					<span>Dark</span>
				</DropdownMenuItem>
				<DropdownMenuItem
					onClick={() => setTheme("system")}
					className="flex items-center space-x-2"
				>
					<Laptop className="h-4 w-4" />
					<span>System</span>
				</DropdownMenuItem>
			</DropdownMenuContent>
		</DropdownMenu>
	);
}
