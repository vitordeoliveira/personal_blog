'use client';

import { motion } from "framer-motion";
import { Bot, Building2, BookOpen, Headphones, Users, ExternalLink } from "lucide-react";
import { Button } from "./ui/button";

interface HeroProps {
  onOpenChat: () => void;
}

export function Hero({ onOpenChat }: HeroProps) {
  const products = [
    { icon: Users, label: "Sales Agent Platform", description: "AI-powered sales automation" },
    { icon: BookOpen, label: "Histories Books Generator", description: "Create personalized stories" },
    { icon: Headphones, label: "Support Agents", description: "Intelligent customer support" },
  ];

  return (
    <section className="relative py-16 md:py-24 max-w-5xl mx-auto overflow-visible">
      {/* Glow effect */}
      <div 
        className="absolute inset-0 z-[1] bg-glow opacity-70 pointer-events-none w-full" 
        style={{ 
          backgroundImage: 'var(--gradient-glow)',
          backgroundRepeat: 'no-repeat',
          backgroundPosition: 'center',
          backgroundSize: 'cover'
        }} 
      />
      
      <div className="container relative z-[2]">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="max-w-3xl"
        >
          <motion.p
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.1 }}
            className="text-sm font-medium text-action-primary mb-4"
          >
            10+ Years Experience • JavaScript • Rust • AI
          </motion.p>

          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="text-4xl md:text-5xl lg:text-6xl font-extrabold tracking-tight mb-6"
          >
            Hi, I'm{" "}
            <span className="text-gradient">Vitor de Oliveira</span>
          </motion.h1>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="text-lg md:text-xl text-text-muted leading-relaxed mb-6"
          >
            With over 10+ years of experience in the tech industry, I've had the
            privilege of working with leading companies like{" "}
            <span className="text-text font-medium">Volkswagen</span>, where I've
            contributed to innovative projects and cutting-edge solutions.
          </motion.p>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.4 }}
            className="text-text-muted leading-relaxed mb-8"
          >
            I specialize in JavaScript, Rust, and AI, combining these technologies to
            build robust, performant, and intelligent applications. Beyond my technical
            expertise, I'm also an entrepreneur—founder of{" "}
            <span className="text-text font-medium">3 companies</span> and partner in another.
          </motion.p>

          {/* Invisible Matrix Products */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5 }}
            className="mb-8"
          >
            <div className="flex items-center gap-2 mb-4">
              <Building2 className="w-4 h-4 text-action-primary" />
              <span className="text-sm font-medium text-text-muted">Founder of</span>
              <a 
                href="https://invisible-matrix.com" 
                target="_blank" 
                rel="noopener noreferrer"
                className="text-sm font-semibold text-action-primary hover:underline inline-flex items-center gap-1"
              >
                Invisible Matrix
                <ExternalLink className="w-3 h-3" />
              </a>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              {products.map((item, index) => (
                <motion.div
                  key={item.label}
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: 0.5 + index * 0.1 }}
                  className="flex items-center gap-3 p-4 rounded-xl bg-bg-light border border-border/50 hover:border-action-primary/30 hover:shadow-glow transition-all duration-300"
                >
                  <div className="w-10 h-10 rounded-lg bg-action-primary/10 flex items-center justify-center">
                    <item.icon className="w-5 h-5 text-action-primary" />
                  </div>
                  <div>
                    <p className="font-semibold text-sm">{item.label}</p>
                    <p className="text-xs text-text-muted">{item.description}</p>
                  </div>
                </motion.div>
              ))}
            </div>
          </motion.div>

          {/* CTA Buttons */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.7 }}
            className="flex flex-wrap items-center gap-4 text-action-secondary"
          >
            <Button
              onClick={onOpenChat}
              size="lg"
              className="bg-gradient-primary shadow-glow hover:shadow-glow hover:shadow-2xl hover:scale-105 transition-all duration-300 gap-2"
            >
              <Bot className="w-5 h-5" />
              Talk to my AI
            </Button>
            
            <div className="flex flex-wrap gap-2">
              {["Founder & Partner", "Software Engineer", "AI Enthusiast"].map((role) => (
                <span
                  key={role}
                  className="px-4 py-2 rounded-full text-sm font-medium bg-action-primary/10 text-action-primary"
                >
                  {role}
                </span>
              ))}
            </div>
          </motion.div>
        </motion.div>
      </div>
    </section>
  );
}