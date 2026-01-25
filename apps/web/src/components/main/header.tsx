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
		<div className="flex gap-3 items-center mx-7 lg:mx-0">
			<Skeleton className="size-12 lg:size-12.5 rounded-full" />
			<div className="flex flex-col gap-1 lg:gap-2">
				<Skeleton className="h-5 lg:h-6 w-28 lg:w-32" />
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
					<span className="text-xl sm:text-[28px] leading-[86%] font-medium">
						Stacks Wars
					</span>
				</Link>

				{/* Desktop Navigation */}
				<nav className="hidden lg:flex items-center gap-x-10 text-2xl/8 font-medium">
					{navItems.map((item) => {
						const isActive = pathname.startsWith(item.href);
						return (
							<Link
								key={item.href}
								href={item.href}
								className={cn(
									"transition-colors hover:text-primary",
									isActive
										? "font-semibold text-foreground"
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
							className="flex gap-3 items-center max-w-75 w-full truncate"
						>
							<Avatar className="size-12.5">
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
									<p className="text-base/4 text-foreground/53">
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
							<SheetTitle className="font-medium text-xl leading-[85%]">
								Stacks Wars
							</SheetTitle>
						</SheetHeader>

						{/* Mobile Navigation */}
						<nav className="flex flex-col gap-10 ml-7">
							{navItems.map((item) => {
								const isActive = pathname.startsWith(item.href);
								return (
									<Link
										key={item.href}
										href={item.href}
										onClick={() => setOpen(false)}
										className={cn(
											"text-xl font-medium transition-colors hover:text-primary",
											isActive
												? "font-semibold text-foreground"
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
									className="flex gap-3 items-center mx-7 max-w-75 w-full truncate"
								>
									<Avatar className="size-12">
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
											<p className="text-sm text-foreground/53">
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
								<div className="flex flex-col gap-6 mx-7">
									<Button
										className="rounded-full w-full"
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
										className="rounded-full w-full"
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
