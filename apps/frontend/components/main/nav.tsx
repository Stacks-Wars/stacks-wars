"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";

type NavItem = {
	href: string;
	label: string;
};

type NavProps = {
	items: NavItem[];
	className?: string;
};

export default function Nav({ items, className }: NavProps) {
	const pathname = usePathname();

	return (
		<nav className={cn("flex items-center justify-between", className)}>
			{items.map((item) => {
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
	);
}
