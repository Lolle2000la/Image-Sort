namespace ImageSort.Actions
{
    public interface IReversibleAction
    {
        string DisplayName { get; }
        void Act();

        void Revert();
    }
}