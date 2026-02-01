"use client";

import { useState } from "react";
import type { Route } from "next";
import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { usePathname } from "next/navigation";
import { useUser, useUserLoading } from "@/lib/stores/user";
import { MenuIcon } from "lucide-react";
import {
	Sheet,
	SheetContent,
	SheetHeader,
	SheetTitle,
	SheetTrigger,
} from "@/components/ui/sheet";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Skeleton } from "@/components/ui/skeleton";
import { formatAddress } from "@/lib/utils";

const navItems: { href: Route; label: string }[] = [
	{ href: "/games", label: "Games" },
	{ href: "/lobby", label: "Lobby" },
	{ href: "/leaderboard", label: "Leaderboard" },
];

export default function Header() {
	const pathname = usePathname();
	const user = useUser();
	const isLoading = useUserLoading();
	const [open, setOpen] = useState(false);

	const isAuthenticated = !isLoading && user;

	const AuthSkeleton = () => (
		<div className="mx-7 flex items-center gap-3 lg:mx-0">
			<Skeleton className="size-12 rounded-full lg:size-12.5" />
			<div className="flex flex-col gap-1 lg:gap-2">
				<Skeleton className="h-5 w-28 lg:h-6 lg:w-32" />
				<Skeleton className="h-4 w-20 lg:w-24" />
			</div>
		</div>
	);

	return (
		<header className="container mx-auto px-4">
			<div className="flex items-center justify-between gap-4 py-6">
				<Link href={"/"} className="flex items-center gap-3 sm:gap-4">
					<Image
						src={"/logo.svg"}
						alt="stacks wars logo"
						height={51}
						width={51}
						className="size-9.5 sm:size-12.5"
					/>
					<span className="text-xl leading-[86%] font-medium sm:text-[28px]">
						Stacks Wars
					</span>
				</Link>

				{/* Desktop Navigation */}
				<nav className="hidden items-center gap-x-10 text-2xl/8 font-medium lg:flex">
					{navItems.map((item) => {
						const isActive = pathname.startsWith(item.href);
						return (
							<Link
								key={item.href}
								href={item.href}
								className={cn(
									"hover:text-primary transition-colors",
									isActive
										? "text-foreground font-semibold"
										: "text-foreground/40"
								)}
							>
								{item.label}
							</Link>
						);
					})}
				</nav>

				{/* Desktop Profile/Auth */}
				<div className="hidden lg:block">
					{isLoading ? (
						<AuthSkeleton />
					) : isAuthenticated ? (
						<Link
							href={`/u/${user.username || user.walletAddress}`}
							className="flex w-full max-w-75 items-center gap-3 truncate"
						>
							<Avatar className="size-12.5 border">
								<AvatarImage
									//src={"/images/avatar.svg"}
									alt="profile photo"
									width={50}
									height={50}
								/>
								<AvatarFallback>
									{(
										user.displayName ||
										user.username ||
										user.walletAddress
									)
										.slice(0, 2)
										.toUpperCase()}
								</AvatarFallback>
							</Avatar>
							{user.displayName ? (
								<div className="flex flex-col gap-2">
									<p className="text-2xl/6">
										{user.displayName}
									</p>
									<p className="text-foreground/53 text-base/4">
										{user.username ||
											formatAddress(user.walletAddress)}
									</p>
								</div>
							) : (
								<p className="text-2xl/6">
									{user.username ||
										formatAddress(user.walletAddress)}
								</p>
							)}
						</Link>
					) : (
						<div className="flex items-center gap-4">
							<Button className="rounded-full" asChild>
								<Link href={"/signup"}>Create an Account</Link>
							</Button>
							<Button
								variant={"outline"}
								className="rounded-full"
								asChild
							>
								<Link href={"/login"}>Login</Link>
							</Button>
						</div>
					)}
				</div>

				{/* Mobile Menu */}
				<Sheet open={open} onOpenChange={setOpen}>
					<SheetTrigger asChild className="lg:hidden">
						<Button variant="ghost" size="icon">
							<MenuIcon className="size-8" />
							<span className="sr-only">Toggle menu</span>
						</Button>
					</SheetTrigger>
					<SheetContent side="right" className="w-90 gap-10">
						<SheetHeader>
							<SheetTitle className="text-xl leading-[85%] font-medium">
								Stacks Wars
							</SheetTitle>
						</SheetHeader>

						{/* Mobile Navigation */}
						<nav className="ml-7 flex flex-col gap-10">
							{navItems.map((item) => {
								const isActive = pathname.startsWith(item.href);
								return (
									<Link
										key={item.href}
										href={item.href}
										onClick={() => setOpen(false)}
										className={cn(
											"hover:text-primary text-xl font-medium transition-colors",
											isActive
												? "text-foreground font-semibold"
												: "text-foreground/40"
										)}
									>
										{item.label}
									</Link>
								);
							})}
						</nav>

						{/* Mobile Profile/Auth */}
						<div className="border-t pt-10">
							{isLoading ? (
								<AuthSkeleton />
							) : isAuthenticated ? (
								<Link
									href={`/u/${user.username || user.walletAddress}`}
									onClick={() => setOpen(false)}
									className="mx-7 flex w-full max-w-75 items-center gap-3 truncate"
								>
									<Avatar className="size-12 border">
										<AvatarImage
											//src={"/images/avatar.svg"}
											alt="profile photo"
											width={48}
											height={48}
										/>
										<AvatarFallback>
											{(
												user.displayName ||
												user.username ||
												user.walletAddress
											)
												.slice(0, 2)
												.toUpperCase()}
										</AvatarFallback>
									</Avatar>
									{user.displayName ? (
										<div className="flex flex-col gap-1">
											<p className="text-lg font-medium">
												{user.displayName}
											</p>
											<p className="text-foreground/53 text-sm">
												{user.username ||
													formatAddress(
														user.walletAddress
													)}
											</p>
										</div>
									) : (
										<p className="text-lg font-medium">
											{user.username ||
												formatAddress(
													user.walletAddress
												)}
										</p>
									)}
								</Link>
							) : (
								<div className="mx-7 flex flex-col gap-6">
									<Button
										className="w-full rounded-full"
										asChild
									>
										<Link
											href={"/signup"}
											onClick={() => setOpen(false)}
										>
											Create an Account
										</Link>
									</Button>
									<Button
										variant={"outline"}
										className="w-full rounded-full"
										asChild
									>
										<Link
											href={"/login"}
											onClick={() => setOpen(false)}
										>
											Login
										</Link>
									</Button>
								</div>
							)}
						</div>
					</SheetContent>
				</Sheet>
			</div>
		</header>
	);
}
