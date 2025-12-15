import type { Route } from "next";
import Image from "next/image";
import Link from "next/link";
import Nav from "@/components/main/nav";
import { Button } from "@/components/ui/button";

const navItems: { href: Route; label: string }[] = [
	{ href: "/games", label: "Games" },
	{ href: "/leaderboard", label: "Leaderboard" },
	{ href: "/lobby", label: "Lobby" },
];

export default function Header() {
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
				<Nav
					items={navItems}
					className="w-full max-w-md text-2xl font-medium"
				/>
				<div className="flex items-center gap-4 rounded-full text-sm font-medium">
					<Button className="" asChild>
						<Link href={"/signup"}>Create an Account</Link>
					</Button>
					<Button variant={"outline"} asChild>
						<Link href={"/login"}>Login</Link>
					</Button>
				</div>
			</div>
		</header>
	);
}
