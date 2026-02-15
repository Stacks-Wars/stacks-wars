import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function Hero() {
	return (
		<div className="min-h-[50vh] lg:min-h-[60vh] relative flex items-center justify-center py-16 lg:py-26 px-4 w-full bg-[#312F2D] flex-col gap-3 rounded-3xl md:rounded-4xl overflow-hidden">
			{/* Hero Banner Background */}
			<Image
				className="absolute inset-0 w-full h-full object-cover z-0"
				src="/images/hero-banner.png"
				fill
				sizes="100vw"
				alt=""
				priority
			/>

			{/* Optional overlay for better text readability */}
			<div className="absolute inset-0 bg-black/30 z-0" />

			{/* Content */}
			<div className="relative z-10 flex flex-col items-center gap-3">
				<h3 className="capitalize text-3xl md:text-5xl lg:text-8xl font-bold">
					battle of words
				</h3>
				<p className="font-medium md:text-xl lg:text-2xl max-w-3xl text-center px-4">
					Experience the thrill of Stacks Wars with our first game. Dive in,
					test your skills, and claim victory!
				</p>
			</div>

			{/* Ruby Images */}
			<Image
				className="absolute scale-x-[-1] -top-6 min-[500px]:-top-16 left-[31.33px] min-[500px]:w-40 min-[500px]:h-40 md:w-72 md:h-72 object-contain shrink-0 z-10"
				src="/images/ruby1.png"
				width={25.9}
				height={30.9}
				sizes="100vw"
				alt=""
			/>
			<Image
				className="absolute top-[50%] -right-2 md:-right-20 min-[500px]:-right-16 min-[500px]:w-40 min-[500px]:h-40 md:w-72 md:h-72 object-cover shrink-0 z-10"
				width={35}
				src="/images/ruby1.png"
				height={42}
				sizes="100vw"
				alt=""
			/>

			{/* Button */}
			<Button
				variant="outline"
				className="absolute -bottom-5 -translate-y-1/2 w-full max-w-48 sm:max-w-52 lg:max-w-80 rounded-full text-sm sm:text-base lg:text-xl font-medium -mb-4 sm:-mb-6 lg:-mb-8 py-3 sm:py-3.5 lg:py-4 h-8 sm:h-12 lg:h-16 shadow-sm z-10"
				asChild
			>
				<Link href="/games">Play Now!</Link>
			</Button>
		</div>
	);
}
