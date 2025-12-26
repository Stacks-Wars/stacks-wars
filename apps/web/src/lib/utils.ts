import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export function formatAddress(
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
