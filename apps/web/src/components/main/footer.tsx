"use client";

import Image from "next/image";
import { Button } from "@/components/ui/button";

const FooterImg = "/images/footer-img.svg";
const FooterText = "/images/footer-text.svg";

export default function Footer() {
	return (
		<footer className="w-full py-8 mt-16 md:mt-32">
			<div className="container mx-auto flex flex-col w-full items-center justify-between gap-2 md:gap-8">
				<div className="w-full mb-12 md:mb-38">
					<Image
						src={FooterImg}
						alt="Footer Illustration"
						width={1920}
						height={120}
						className="w-full h-12 md:h-auto object-cover"
						priority
					/>
				</div>
				<h2 className="text-center text-3xl md:text-5xl font-bold text-foreground mb-2 md:mb-4">
					Every stack counts. Every move matters. Are you ready?
				</h2>
				<p className="text-center text-2xl md:text-3xl text-foreground font-medium">
					Join the community
				</p>
				<div className="flex flex-row md:flex-row gap-4 md:w-auto justify-center items-center">
					<Button
						asChild
						variant="outline"
						size="lg"
						className="cursor-pointer rounded-full p-6"
					>
						<a
							href="https://x.com/stacks-wars"
							target="_blank"
							rel="noopener noreferrer"
						>
							X (formerly Twitter)
						</a>
					</Button>
					<Button
						asChild
						variant="outline"
						size="lg"
						className="cursor-pointer rounded-full p-6"
					>
						<a
							href="https://t.me/stacks-wars"
							target="_blank"
							rel="noopener noreferrer"
						>
							Telegram
						</a>
					</Button>
				</div>

				<div className="w-full">
					<Image
						src={FooterText}
						alt="Footer Text"
						width={1920}
						height={120}
						className="hidden md:block w-full h-auto object-cover mt-28 bottom-0"
						priority
					/>
				</div>
			</div>
		</footer>
	);
}
