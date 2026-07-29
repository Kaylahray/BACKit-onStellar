import type { Metadata } from "next";
import { homeMetadata } from "@/lib/metadata";
import LandingClient from "./landing-client";

export const metadata: Metadata = homeMetadata;

export default function HomePage() {
  return <LandingClient />;
}
