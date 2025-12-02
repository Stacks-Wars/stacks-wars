import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export const truncateAddress = (address: string | undefined): string => {
	if (!address) return "";
	return `${address.slice(0, 6)}...${address.slice(-4)}`;
};

export const formatNumber = (num: number, decimals: number = 1): string => {
	if (num === 0) return "0";

	const k = 1000;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes = ["", "K", "M", "B", "T"];

	const i = Math.floor(Math.log(Math.abs(num)) / Math.log(k));

	if (i === 0) {
		// For numbers less than 1000, show up to 2 decimal places if they're not whole numbers
		return num % 1 === 0 ? num.toString() : num.toFixed(Math.min(2, dm));
	}

	const formattedNumber = parseFloat((num / Math.pow(k, i)).toFixed(dm));

	// Remove unnecessary trailing zeros
	const cleanNumber =
		formattedNumber % 1 === 0
			? Math.floor(formattedNumber).toString()
			: formattedNumber.toString();

	return cleanNumber + sizes[i];
};
