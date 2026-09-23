# Copied from the Polkadot Fellowship's `/cmd` bot:
# https://github.com/polkadot-fellows/runtimes/blob/1eb4d2e30e016cb965a2dabab6b0e2e8efb39fd4/.github/scripts/cmd/_help.py
# Copyright (C) the Polkadot Fellowship and contributors; licensed under GPL-3.0.
# SPDX-License-Identifier: GPL-3.0-only

import argparse

"""

Custom help action for argparse, it prints the help message for the main parser and all subparsers.

"""


class _HelpAction(argparse._HelpAction):
    def __call__(self, parser, namespace, values, option_string=None):
        parser.print_help()

        # retrieve subparsers from parser
        subparsers_actions = [
            action for action in parser._actions
            if isinstance(action, argparse._SubParsersAction)]
        # there will probably only be one subparser_action,
        # but better save than sorry
        for subparsers_action in subparsers_actions:
            # get all subparsers and print help
            for choice, subparser in subparsers_action.choices.items():
                print("\n### Command '{}'".format(choice))
                print(subparser.format_help())

        parser.exit()
