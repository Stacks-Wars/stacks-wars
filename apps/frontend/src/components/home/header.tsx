"use client";
import { Button } from "@/components/ui/button";
import {
	Sheet,
	SheetContent,
	SheetHeader,
	SheetTitle,
	SheetTrigger,
} from "@/components/ui/sheet";
import Image from "next/image";
import Link from "next/link";
import { FiMenu } from "react-icons/fi";
import { ThemeToggle } from "../theme/theme-toggle";
import ConnectWallet from "./connect-wallet";
import { Loader, LogOut, UserIcon, Wallet2 } from "lucide-react";
import { useConnectUser } from "@/contexts/ConnectWalletContext";
import { useState } from "react";

export default function Header() {
	const navLinks = [
		//{ href: "/", label: "Home" },
		{ href: "/games", label: "Games" },
		{ href: "/lobby", label: "Lobby" },
		{ href: "/leaderboard", label: "Leaderboard" },
	];

	const { isConnected, isConnecting, user, handleConnect, handleDisconnect } =
		useConnectUser();

	const [openSheet, setSheetOpen] = useState(false);

	return (
		<header className="supports-[backdrop-filter]:bg-primary/30 sticky top-0 z-50 w-full border-b backdrop-blur">
			<div className="max-w-8xl mx-auto flex h-16 items-center justify-between px-6">
				<Link href={"/"}>
					<div className="flex items-center gap-2">
						<Image
							src="/logo.webp?height=32&width=32"
							alt="Stacks Wars Logo"
							width={32}
							height={32}
							className="rounded-md"
						/>
						<span className="hidden text-xl font-bold md:inline-block">
							Stacks Wars
						</span>
						<span className="text-xl font-bold md:hidden">SW</span>
					</div>
				</Link>

				{/* Desktop Navigation */}
				<nav className="hidden items-center gap-6 md:flex">
					{navLinks.map((link) => (
						<Link
							key={link.href}
							href={link.href}
							className="hover:text-primary text-sm font-medium transition-colors"
						>
							{link.label}
						</Link>
					))}
				</nav>

				<div className="flex items-center gap-4 md:hidden">
					<Sheet open={openSheet} onOpenChange={setSheetOpen}>
						<SheetTrigger asChild className="md:hidden">
							<Button variant="ghost" size="icon">
								<FiMenu className="size-8" />
								<span className="sr-only">Toggle menu</span>
							</Button>
						</SheetTrigger>
						<SheetContent
							side="right"
							className="supports-[backdrop-filter]:bg-primary/30 backdrop-blur"
						>
							<SheetHeader>
								<SheetTitle>Stacks Wars</SheetTitle>
							</SheetHeader>
							<nav className="mt-6 flex flex-col gap-4 px-4">
								{navLinks.map((link) => (
									<Button
										key={link.href}
										variant={"ghost"}
										asChild
										className="flex justify-start gap-2 text-sm"
										onClick={() => setSheetOpen(false)}
									>
										<Link href={link.href}>
											{link.label}
										</Link>
									</Button>
								))}
								<ThemeToggle className="justify-start" />
								{!isConnected ? (
									<Button
										variant="ghost"
										onClick={() => {
											handleConnect();
											setSheetOpen(false);
										}}
										disabled={isConnecting}
										className="justify-start text-sm"
									>
										{isConnecting ? (
											<Loader className="mr-2 h-4 w-4 animate-spin" />
										) : (
											<Wallet2 className="mr-2 h-4 w-4" />
										)}
										{isConnecting
											? "Connecting..."
											: "Connect wallet"}
									</Button>
								) : (
									<>
										<Button
											variant={"ghost"}
											asChild
											className="flex items-center justify-start gap-2 px-0 text-sm"
											onClick={() => {
												setSheetOpen(false);
											}}
										>
											<Link
												href={`/u/${user?.username || user?.walletAddress}`}
											>
												<UserIcon className="h-4 w-4" />
												Profile
											</Link>
										</Button>
										<Button
											variant="ghost"
											onClick={() => {
												handleDisconnect();
												setSheetOpen(false);
											}}
											disabled={isConnecting}
											className="text-destructive flex items-center justify-start gap-2 px-0 text-sm"
										>
											{isConnecting ? (
												<Loader className="mr-2 h-4 w-4 animate-spin" />
											) : (
												<LogOut className="mr-2 h-4 w-4" />
											)}
											Disconnect
										</Button>
									</>
								)}
							</nav>
						</SheetContent>
					</Sheet>
				</div>
				<div className="hidden md:flex">
					<ConnectWallet />
					<ThemeToggle />
				</div>
			</div>
		</header>
	);
}
