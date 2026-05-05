# swcuors

ClassicUO plugin for the Freeshard [Schattenwelt](https://alte-schattenwelt.de)

## Features

- Typing Indicator

  Sends a speech packet with text `[typing]` to the server. This will be filtered out server side and replaced with some effect animation. E.g:

  ```
        public override void OnSaid(SpeechEventArgs e)
        {
	        if ( e.Speech == "[typing]" )
	        {
		        e.Handled = true;
		        e.Blocked = true;
		        PlayTypingAnimation();
	        }
          base.OnSaid(e);
        }

        private Timer m_LastPlayedTypingAnimationTimer = null;
        public void StartTypingAnimationTimer()
        {
	        m_LastPlayedTypingAnimationTimer?.Stop();
	        m_LastPlayedTypingAnimationTimer = Timer.DelayCall( TimeSpan.FromMilliseconds( 10 ), TimeSpan.FromMilliseconds(200), 20, new TimerStateCallback(TypeAnimation), new TypingAnimation(this) );
	        m_LastPlayedTypingAnimationTimer.Start();
        }

        private class TypingAnimation( Mobile mobile )
        {
	        public Mobile Mobile { get; set; } = mobile;
	        public int Count { get; set; } = 0;
        }
        
        private static void TypeAnimation( object o )
        {
	        if ( o is TypingAnimation animInfo && animInfo.Mobile != null && !animInfo.Mobile.Hidden )
	        {
		        int height = animInfo.Mobile.Z + 18;
		        Entity ent = new Entity(Serial.Zero, new Point3D(animInfo.Mobile.X, animInfo.Mobile.Y, height), animInfo.Mobile.Map);
		        Effects.SendTargetParticles(ent, 0x1810 + animInfo.Count, 1, 5, animInfo.Mobile.SpeechHue, EffectLayer.Head, 0);
				animInfo.Count++;
				if (animInfo.Count > 12) animInfo.Count = 0;
	        };
        }
  ```
