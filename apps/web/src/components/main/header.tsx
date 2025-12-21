"use client";

import type { Route } from "next";
import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { usePathname } from "next/navigation";
import { useUserStore } from "@/lib/stores/user";

const navItems: { href: Route; label: string }[] = [
	{ href: "/games", label: "Games" },
	{ href: "/leaderboard", label: "Leaderboard" },
	{ href: "/lobby", label: "Lobby" },
];

function formatAddress(
	address: string,
	options: {
		length?: number;
		separator?: string;
	} = {}
): string {
	const { length = 4, separator = "..." } = options;

	const start = address.slice(0, length);
	const end = address.slice(-length);
	return `${start}${separator}${end}`;
}

export default function Header() {
	const pathname = usePathname();
	const { user } = useUserStore();

	return (
		<header className="container mx-auto">
			<div className="flex items-center justify-between gap-4 py-6">
				<Link href={"/"} className="flex items-center gap-4">
					<Image
						src={"/logo.svg"}
						alt="stacks wars logo"
						height={51}
						width={51}
					/>
					<span className="text-3xl font-medium">Stacks Wars</span>
				</Link>
				<nav
					className={
						"flex items-center justify-between w-full max-w-md text-2xl font-medium"
					}
				>
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
										: "text-foreground/60"
								)}
							>
								{item.label}
							</Link>
						);
					})}
				</nav>
				{user ? (
					<div className="flex gap-3 items-center">
						<Image
							src={"/images/avatar.svg"}
							alt="profile image"
							width={60}
							height={60}
						/>
						{user.displayName ? (
							<div className="flex flex-col gap-2">
								<p className="text-2xl/6">{user.displayName}</p>
								<p className="text-base/4 text-foreground/53">
									{user.username ||
										formatAddress(user.walletAddress)}
								</p>
							</div>
						) : user.username ? (
							<div className="flex flex-col gap-2">
								<p className="text-2xl/6">user.username</p>
								<p className="text-base/4 text-foreground/53">
									{formatAddress(user.walletAddress)}
								</p>
							</div>
						) : (
							<p className="text-2xl/6">
								{formatAddress(user.walletAddress)}
							</p>
						)}
					</div>
				) : (
					<div className="flex items-center gap-4 rounded-full text-sm font-medium">
						<Button className="" asChild>
							<Link href={"/signup"}>Create an Account</Link>
						</Button>
						<Button variant={"outline"} asChild>
							<Link href={"/login"}>Login</Link>
						</Button>
					</div>
				)}
			</div>
		</header>
	);
}
